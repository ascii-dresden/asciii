//! Project file parsing and evaluation.
//!
//! This module implements all functionality of a project.

use std::fs::File;
use std::io::prelude::*;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::error::Error as ErrorTrait;
use std::collections::HashMap;

use chrono::prelude::*;
use chrono::Duration;
use yaml_rust::Yaml;
#[cfg(feature="serialization")] use serde_json;
use slug;
use tempdir::TempDir;

use bill::BillItem;
use icalendar::*;
use semver::Version;

use super::BillType;
use util::{yaml, get_valid_path};
use storage::{Storable, StorageResult, list_path_content};
use storage::ErrorKind as StorageErrorKind;
use storage::repo::GitStatus;
use templater::{Templater, IsKeyword};

pub mod product;
pub mod spec;
mod spec_yaml;

pub mod error;
mod computed_field;

#[cfg(feature="deserialization")] pub mod import;
#[cfg(feature="serialization")] pub mod export;
#[cfg(feature="serialization")] use self::export::*;

use self::spec::{IsProject, IsClient};
use self::spec::{Offerable, Invoicable, Redeemable, Validatable, HasEmployees};
use self::spec_yaml::ProvidesData;

use self::error::{ErrorKind, ErrorList, SpecResult, Result};
use self::product::Product;
use self::product::error as product_error;

pub use self::computed_field::ComputedField;

/// Represents a Project.
///
/// A project is storable, contains products, and you can create an offer or invoice from it.
/// The main implementation is done in [`spec`](spec/index.html).
pub struct Project {
    file_path: PathBuf,
    _temp_dir: Option<TempDir>,
    git_status: Option<GitStatus>,
    file_content: String,
    yaml: Yaml
}

impl Project {
    /// Access to inner data
    pub fn yaml(&self) -> &Yaml{ &self.yaml }

    /// Opens a project from file path;
    pub fn open<S: AsRef<OsStr> + ?Sized>(pathish: &S) -> Result<Project> {
        let file_path = Path::new(&pathish);
        let file_content = File::open(&file_path)
                                .and_then(|mut file| {
                                    let mut content = String::new();
                                    file.read_to_string(&mut content).map(|_| content)
                                })?;
        Ok(Project {
            file_path: file_path.to_owned(),
            _temp_dir: None,
            git_status: None,
            yaml: yaml::parse(&file_content).unwrap_or_else(|e|{
                error!("syntax error in {}\n  {}", file_path.display(), e);
                Yaml::Null
            }),
            file_content: file_content,
        })
    }

    /// import from yaml file
    #[cfg(feature="deserialization")]
    pub fn parse_yaml(&self) -> Result<import::Project> {
        import::from_str(&self.file_content)
    }

    /// (feature deactivated) import from yaml file
    #[cfg(not(feature="deserialization"))]
    pub fn parse_yaml(&self) -> Result<()> {
        bail!(error::ErrorKind::FeatureDeactivated)
    }

    pub fn dump_yaml(&self) -> String {
        use yaml_rust::emitter::YamlEmitter;
        let mut buf = String::new();
        {
            let mut emitter = YamlEmitter::new(&mut buf);
            emitter.dump(self.yaml()).unwrap();
        }
        buf
    }

    #[cfg(feature="serialization")]
    /// export to JSON
    pub fn to_json(&self) -> Result<String> {
        let complete: Complete = self.export();
        Ok(serde_json::to_string(&complete)?)
    }

    #[cfg(not(feature="serialization"))]
    /// feature deactivateda) export to JSON
    pub fn to_json(&self) -> Result<String> {
        bail!(error::ErrorKind::FeatureDeactivated)
    }

    /// Used mostly for testing purposes
    pub fn from_file_content(content: &str) -> Result<Project> {
        Ok(Project{
            file_path: PathBuf::new(),
            _temp_dir: None,
            git_status: None,
            yaml: yaml::parse(&content).unwrap(),
            file_content: String::from(content),
        })
    }

    /// wrapper around yaml::get() with replacement
    pub fn field(&self, path:&str) -> Option<String> {
        ComputedField::from(path).get(self).or_else(||
            yaml::get_to_string(self.yaml(),path)
        )
    }

    /// Returns the struct `Client`, which abstracts away client specific stuff.
    pub fn client(&self) -> Client {
        Client { inner: self }
    }

    /// Returns the struct `Offer`, which abstracts away offer specific stuff.
    pub fn offer(&self) -> Offer {
        Offer { inner: self }
    }

    /// Returns the struct `Invoice`, which abstracts away invoice specific stuff.
    pub fn invoice(&self) -> Invoice {
        Invoice { inner: self }
    }

    /// Returns the struct `Invoice`, which abstracts away invoice specific stuff.
    pub fn hours(&self) -> Hours {
        Hours { inner: self }
    }

    /// Ready to produce offer.
    ///
    /// Ready to send an **offer** to the client.
    pub fn is_ready_for_offer(&self) -> SpecResult{
        self::error::combine_specresults(
            vec![ self.offer().validate(),
                  self.client().validate(),
                  self.validate() ]
            )
    }

    /// Valid to produce invoice
    ///
    /// Ready to send an **invoice** to the client.
    pub fn is_ready_for_invoice(&self) -> SpecResult{
        self::error::combine_specresults(
            vec![ self.is_ready_for_offer(),
                  self.invoice().validate()]
            )
    }

    /// Completely done and in the past.
    ///
    /// Ready to be **h:
    pub fn is_ready_for_archive(&self) -> SpecResult {
        if self.canceled(){
            Ok(())
        } else {
            self::error::combine_specresults(
                vec![ Redeemable::validate(self),
                      self.hours().validate() ]
                )
        }
    }

    pub fn to_csv(&self, bill_type:&BillType) -> Result<String>{
        use std::fmt::Write;
        let (offer, invoice) = self.bills()?;
        let bill = match *bill_type{ BillType::Offer => offer, BillType::Invoice => invoice };
        let mut csv_string = String::new();
        let splitter = ";";

        writeln!(&mut csv_string, "{}", [ "#", "Bezeichnung", "Menge", "EP", "Steuer", "Preis"].join(splitter))?;


        for items in bill.items_by_tax.values(){
            for (index,item) in items.iter().enumerate(){
                                        write!(&mut csv_string, "{};",  &index.to_string())?;
                                        write!(&mut csv_string, "{};",  item.product.name)?;
                                        write!(&mut csv_string, "{};",  item.amount.to_string())?;
                                        write!(&mut csv_string, "{:.2};",  item.product.price.as_float())?;
                                        write!(&mut csv_string, "{:.2};",  item.product.tax)?;
                                        write!(&mut csv_string, "{:.2}\n", (item.product.price * item.amount).as_float())?;
            }
        }
        Ok(csv_string)
    }

    pub fn debug(&self) -> Debug {
        self.into()
    }

    /// Check Templated for replacable markers
    pub fn empty_fields(&self) -> Vec<String>{
        self.file_content.list_keywords()
    }

    /// Fill certain field
    pub fn replace_field(&self, field:&str, value:&str) -> Result<()> {
        // fills the template
        let filled = Templater::new(&self.file_content)
            .fill_in_field(field,value)
            .finalize()
            .filled;

        match yaml::parse(&filled){
            Ok(_) => {
                let mut file = File::create(self.file())?;
                file.write_all(filled.as_bytes())?;
                file.sync_all()?;
                Ok(())
            },
            Err(e) => {
                error!("The resulting document is no valid yaml. SORRY!\n{}\n\n{}",
                       filled.lines().enumerate().map(|(n,l)| format!("{:>3}. {}\n",n,l)).collect::<String>(), //line numbers :D
                       e.description());
                bail!(e)
            }
        }
    }

    /// Time between event and creation of invoice
    pub fn our_bad(&self) -> Option<Duration> {
        let event   = self.event_date()?;
        let invoice = self.invoice().date().unwrap_or_else(Utc::today);
        let diff = invoice.signed_duration_since(event);
        if diff > Duration::zero() {
            Some(diff)
        } else {
            None
        }
    }

    /// Time between creation of invoice and payment
    pub fn their_bad(&self) -> Option<Duration> {
        let invoice = self.invoice().date().unwrap_or_else(Utc::today);
        let payed   = self.payed_date().unwrap_or_else(Utc::today);
        Some(invoice.signed_duration_since(payed))
    }

    /// What I need to do
    ///
    /// Produces an iCal calendar from this project.
    pub fn to_tasks(&self) -> Calendar {
        //return if self.canceled();
        let mut cal = Calendar::new();

        let event   = self.event_date();
        let invoice = self.invoice().date();
        let payed   = self.payed_date();
        let wages   = self.hours().wages_date();
        let today   = Utc::today();

        let days_since = |date:Date<Utc>| (today.signed_duration_since(date)).num_days();

        if let Some(event) = event {
            match (invoice, payed, wages) {

                // we need to issue an invoice invoice
                (None,          None,          _) if today >= event => { cal.push(self.task_issue_invoice(event)); },
                (None,          None,          _) if today < event => { /* no need to worry yet */ },

                // they haven't payed us yet
                (Some(invoice), None,          _) if days_since(invoice) >= 14 => { cal.push(self.task_follow_up(invoice)); },
                (Some(invoice), None,          _) if days_since(invoice) < 14 => { /* they have 14 days before we complain */ },

                // we need to pay the employees
                (Some(_),       Some(payed),   None) => { cal.push(self.task_pay_employees(payed)); },

                // everything's all set to close this
                (Some(_),       Some(_),       Some(wages)) if days_since(wages) > 7 => { cal.push(self.task_close_project(wages)); },
                _ => {warn!("weird task edgecase in {:?}:\n{:?}", self.file(), (event, invoice, payed, wages) )}
            }
        }
        cal
    }

    fn task_issue_invoice(&self, event_date: Date<Utc>) -> Todo {
        Todo::new().summary(&lformat!("Create an Invoice"))
                   .due((event_date + Duration::days(14)).and_hms(11, 10, 0))
                   .priority(6)
                   .done()
    }

    fn task_pay_employees(&self, payed_date: Date<Utc>) -> Todo {
        let days_since_payed = (Utc::today().signed_duration_since(payed_date)).num_days();
        Todo::new().summary(&lformat!("{}: Hungry employees!", self.invoice().number_str().unwrap_or_else(String::new)))
            .description( &lformat!("Pay {}\nYou have had the money for {} days!",
                                   self.hours().employees_string().unwrap_or_else(String::new),
                                   days_since_payed))
            .due((payed_date + Duration::days(14)).and_hms(11, 10, 0))
            .done()
    }

    fn task_follow_up(&self, invoice_date: Date<Utc>) -> Todo {
        let days_since_invoice = (Utc::today().signed_duration_since(invoice_date)).num_days();
        let mut follow_up = Todo::new();
        follow_up.summary( &lformat!("Inquire about: \"{event}\"!", event = self.name().unwrap()));
        follow_up.description(&lformat!("{inum }{event:?} on {invoice_date} ({days} days ago) was already invoiced but is still not marked as payed.\nPlease check for incoming payments! You can ask {client} ({mail}).",
                                       event = self.name().unwrap(),
                                       days = days_since_invoice,
                                       inum = self.invoice().number_str().unwrap_or_else(String::new),
                                       invoice_date = invoice_date.format("%d.%m.%Y").to_string(),
                                       client = self.client().full_name().unwrap_or_else(String::new),
                                       mail = self.client().email().unwrap_or(""),
                                       ));
        follow_up.priority(3);
        if days_since_invoice > 14 {
            follow_up.summary( &lformat!("{rnum}: payment is {weeks} weeks late: \"{event}\"",
                                        rnum = self.invoice().number_str().unwrap_or_else(String::new),
                                        weeks = days_since_invoice / 7,
                                        event = self.name().unwrap()),
                             );
            follow_up.priority(10);
        }
        follow_up
    }

    fn task_close_project(&self, wages_date: Date<Utc>) -> Todo {
            let days_since_wages = (Utc::today().signed_duration_since(wages_date)).num_days();
            Todo::new().summary( &lformat!("Archive {}", self.name().unwrap()))
                       .description( &lformat!("{:?} has been finished for {} days, get rid of it!",
                                              self.name().unwrap(),
                                              days_since_wages))
                       .done()
    }

    fn item_from_desc_and_value<'y>(&self, desc: &'y Yaml, values: &'y Yaml) -> product_error::Result<(BillItem<Product<'y>>,BillItem<Product<'y>>)> {
        let get_f64 = |yaml, path|
            self.get_direct(yaml,path)
                .and_then(|y| y.as_f64()
                               .or_else(|| y.as_i64()
                                     .map(|y|y as f64)
                                  )
                         );

        let product = Product::from_desc_and_value(desc, values, self.tax())?;

        let offered = get_f64(values, "amount")
                           .ok_or_else(
                               || product_error::ErrorKind::MissingAmount(product.name.to_owned())
                               )?;

        let sold = get_f64(values, "sold");
        // TODO test this
        let sold = if let Some(returned) = get_f64(values, "returned") {
            // if "returned", there must be no "sold"
            if sold.is_some() {
                bail!(product_error::ErrorKind::AmbiguousAmounts(product.name.to_owned()));
            }
            if returned > offered {
                bail!(product_error::ErrorKind::TooMuchReturned(product.name.to_owned()));
            }
            offered - returned
        } else if let Some(sold) = sold {
            sold
        } else {
            offered
        };

        Ok(( BillItem{ amount: offered, product: product }, BillItem{ amount: sold, product: product }))
    }
}

/// Functionality to create output files
pub trait Exportable {
    /// Where to export to
    fn export_dir(&self)  -> PathBuf;

    /// Filename of the offer output file.
    fn offer_file_name(&self, extension:&str) -> Option<String>;

    /// Filename of the invoice output file. **Carefull!** uses today's date.
    fn invoice_file_name(&self, extension:&str) -> Option<String>;

    fn output_file_exists(&self, bill_type: &BillType) -> bool {
        match *bill_type{
            BillType::Offer   => self.offer_file_exists(),
            BillType::Invoice => self.invoice_file_exists()
        }
    }

    fn output_file(&self, bill_type: &BillType) -> Option<PathBuf> {
        match *bill_type{
            BillType::Offer   => self.offer_file(),
            BillType::Invoice => self.invoice_file()
        }
    }

    fn offer_file(&self) -> Option<PathBuf> {
        let output_folder = get_valid_path(::CONFIG.get_str("output_path"));
        let convert_ext  = ::CONFIG.get_str("document_export/output_extension");
        match (output_folder, self.offer_file_name(convert_ext)) {
            (Some(folder), Some(name)) => folder.join(&name).into(),
            _ => None
        }
    }

    fn invoice_file(&self) -> Option<PathBuf>{
        let output_folder = get_valid_path(::CONFIG.get_str("output_path"));
        let convert_ext  = ::CONFIG.get_str("document_export/output_extension");
        match (output_folder, self.invoice_file_name(convert_ext)) {
            (Some(folder), Some(name)) => folder.join(&name).into(),
            _ => None
        }
    }

    fn offer_file_exists(&self) -> bool {
        self.offer_file().map(|f|f.exists()).unwrap_or(false)
    }

    fn invoice_file_exists(&self) -> bool {
        self.invoice_file().map(|f|f.exists()).unwrap_or(false)
    }

    fn write_to_path<P:AsRef<OsStr> + fmt::Debug>(content:&str, target:&P) -> Result<PathBuf> {
        trace!("writing content ({}bytes) to {:?}", content.len(), target);
        let mut file = File::create(Path::new(target))?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        Ok(Path::new(target).to_owned())
    }

    fn write_to_file(&self, content:&str, bill_type:&BillType,ext:&str) -> Result<PathBuf> {
        match *bill_type{
            BillType::Offer   => self.write_to_offer_file(content, ext),
            BillType::Invoice => self.write_to_invoice_file(content, ext)
        }
    }

    fn write_to_offer_file(&self, content:&str, ext:&str) -> Result<PathBuf> {
        if let Some(target) = self.offer_file_name(ext){
            Self::write_to_path(content, &self.export_dir().join(&target))
        } else {bail!(ErrorKind::CantDetermineTargetFile)}
    }

    fn write_to_invoice_file(&self, content:&str, ext:&str) -> Result<PathBuf> {
        if let Some(target) = self.invoice_file_name(ext){
            Self::write_to_path(content, &self.export_dir().join(&target))
        } else {bail!(ErrorKind::CantDetermineTargetFile)}
    }
}

impl Exportable for Project {
    fn export_dir(&self)  -> PathBuf { Storable::dir(self) }

    fn offer_file_name(&self, extension: &str) -> Option<String>{
        let num = self.offer().number()?;
        let name = slug::slugify(IsProject::name(self)?);
        Some(format!("{} {}.{}", num, name, extension))
    }

    fn invoice_file_name(&self, extension: &str) -> Option<String>{
        let num = self.invoice().number_str()?;
        let name = slug::slugify(self.name()?);
        //let date = Local::today().format("%Y-%m-%d").to_string();
        let date = self.invoice().date()?.format("%Y-%m-%d").to_string();
        Some(format!("{} {} {}.{}",num,name,date,extension))
    }

}

impl Storable for Project {
    fn file_extension() -> String {
        ::CONFIG.get_to_string("extensions.project_file")
    }

    fn from_template(project_name:&str,template:&Path, fill: &HashMap<&str,String>) -> StorageResult<Project> {
        let template_name = template.file_stem().unwrap().to_str().unwrap();

        let event_date = (Utc::today() + Duration::days(14)).format("%d.%m.%Y").to_string();
        let created_date = Utc::today().format("%d.%m.%Y").to_string();

        // fill template with these values
        let default_fill = hashmap!{
            "TEMPLATE"      => template_name.to_owned(),
            "PROJECT-NAME"  => project_name.to_owned(),
            "DATE-EVENT"    => event_date,
            "DATE-CREATED"  => created_date,
            "TAX"           => ::CONFIG.get_to_string("defaults/tax"),
            "SALARY"        => ::CONFIG.get_to_string("defaults/salary"),
            "MANAGER"       => ::CONFIG.get_str_or("user/name").unwrap_or("").to_string(),
            "TIME-START"    => String::new(),
            "TIME-END"      => String::new(),
            "VERSION"       => ::VERSION.to_string(),
        };

        // fills the template
        let filled = Templater::from_file(template)?
            .fill_in_data(&fill).fix()
            .fill_in_data(&default_fill)
            .finalize()
            .filled;

        debug!("remaining template fields: {:#?}", filled.list_keywords());

        // generates a temp file
        let temp_dir  = TempDir::new(project_name).unwrap();
        let temp_file = temp_dir.path().join(slug::slugify(project_name) + "." + &Self::file_extension());

        // write into a file
        let mut file = File::create(&temp_file)?;
        file.write_all(filled.as_bytes())?;
        file.sync_all()?;

        let yaml = match yaml::parse(&filled){
            Ok(y) => y,
            Err(e) => {
                error!("The created document is no valid yaml. SORRY!\n{}\n\n{}",
                       filled.lines().enumerate().map(|(n,l)| format!("{:>3}. {}\n",n,l)).collect::<String>(), //line numbers :D
                       e.description());
                bail!(error::Error::from_kind(ErrorKind::Yaml(e)))
            }
        };

        // project now lives in the temp_file
        Ok(Project{
            file_path: temp_file,
            _temp_dir: Some(temp_dir),
            git_status: None,
            file_content: filled,
            yaml: yaml
        })
    }

    fn prefix(&self) -> Option<String>{
        self.invoice().number_str()
    }

    fn index(&self) -> Option<String>{
        let prefix = self.invoice().number_long_str().unwrap_or_else(||String::from("zzzz"));
        match (self.invoice().date(), self.modified_date()) {
            (Some(date), _) |
            (None, Some(date)) => {
                Some(format!("{0}{1}", prefix, date.format("%Y%m%d").to_string()))
            },
            (None, None) => None,
        }
    }

    fn short_desc(&self) -> String {
        self.name()
            .map(|n|n.to_owned())
            .unwrap_or_else(|| format!("unnamed: {:?}",
                                       self.dir()
                                           .file_name()
                                           .expect("the end was \"..\", but why?")
                                      )
                           )
    }

    fn modified_date(&self) -> Option<Date<Utc>> {
        self.get_dmy( "event.dates.0.begin")
            .or_else(||self.get_dmy("created"))
            .or_else(||self.get_dmy("date"))
            // probably the dd-dd.mm.yyyy format
            .or_else(||self.get_str("date")
                           .and_then(|s| yaml::parse_dmy_date_range(s))
                    )
    }

    fn file(&self) -> PathBuf{ self.file_path.to_owned() } // TODO reconsider returning PathBuf at all
    fn set_file(&mut self, new_file:&Path){ self.file_path = new_file.to_owned(); }

    fn set_git_status(&mut self, status:GitStatus){
        self.git_status = Some(status);
    }

    /// Ask a project for its gitstatus
    #[cfg(feature="git_statuses")]
    fn get_git_status(&self) -> GitStatus{
        if let Some(ref status) = self.git_status{
            status.to_owned()
        } else {
            GitStatus::Unknown
        }
    }

    /// Opens a yaml and parses it.
    fn open_folder(folder_path: &Path) -> StorageResult<Project>{
        let project_file_extension = ::CONFIG.get_to_string("extensions.project_file");
        let file_path = list_path_content(folder_path)?.iter()
            .filter(|f|f.extension().unwrap_or(&OsStr::new("")) == project_file_extension.as_str())
            .nth(0).map(|b|b.to_owned())
            .ok_or_else(|| StorageErrorKind::NoProjectFile(folder_path.to_owned()))?;
        Self::open_file(&file_path)
    }

    fn open_file(file_path:&Path) -> StorageResult<Project> {
        Ok(Project::open(file_path)?)
    }

    /// Checks against a certain key-val pair.
    fn matches_filter(&self, key: &str, val: &str) -> bool{
        self.field(key).map_or(false, |c| c.to_lowercase().contains(&val.to_lowercase()))
    }

    /// UNIMPLEMENTED: Checks against a certain search term.
    ///
    /// TODO compare agains InvoiceNumber, ClientFullName, Email, event/name, invoice/official Etc
    fn matches_search(&self, term: &str) -> bool{
        let search = term.to_lowercase();
        self.invoice()
            .number_str()
            .map_or(false, |num|num.to_lowercase().contains(&search))
            ||
            Storable::short_desc(self).to_lowercase().contains(&search)
    }

    fn is_ready_for_archive(&self) -> bool {
        Project::is_ready_for_archive(self).is_ok()
    }
}

/// This is returned by `[Product::client()](struct.Project.html#method.client)`.
pub struct Client<'a> {
    inner: &'a Project
}

/// This is returned by [`Product::offer()`](struct.Project.html#method.offer).
pub struct Offer<'a> {
    inner: &'a Project
}

/// This is returned by [`Product::invoice()`](struct.Project.html#method.invoice).
pub struct Invoice<'a> {
    inner: &'a Project
}

/// This is returned by [`Product::hours()`](struct.Project.html#method.hours).
pub struct Hours<'a> {
    inner: &'a Project
}

/// Output of `Project::debug()`.
///
/// A project is storable, contains products, and you can create an offer or invoice from it.
#[derive(Debug)]
pub struct Debug {
    file_path: PathBuf,
    //temp_dir: Option<PathBuf>, // TODO
    git_status: Option<GitStatus>,
    yaml: Yaml
}

impl<'a> From<&'a Project> for Debug {
    fn from(project: &'a Project) -> Debug {
        Debug {
            file_path:  project.file_path.clone(),
            git_status: project.git_status.clone(),
            yaml:       project.yaml.clone()
        }
    }
}

impl fmt::Debug for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "{:?}", self.debug())
        write!(f, "{:?}{:?}", self.name(), self.file())
    }
}


#[cfg(test)]
mod test {
    use std::path::Path;
    use ::project::spec::*;
    use ::project::Project;
    use ::storage::Storable;

    #[test]
    fn compare_basics(){
        println!("{:?}", ::std::env::current_dir());
        let new_project = Project::open_file(Path::new("./tests/current.yml")).unwrap();
        let old_project = Project::open_file(Path::new("./tests/old.yml")).unwrap();
        //let config = &::CONFIG;

        assert_eq!(old_project.name(),
                   new_project.name());

        assert_eq!(old_project.offer().number(),
                   new_project.offer().number());

        //assert_eq!(old_project.offer().date(), // old format had no offer date
        //           new_project.offer().date());

        assert_eq!(old_project.invoice().number_str(),
                   new_project.invoice().number_str());

        assert_eq!(old_project.invoice().date(),
                   new_project.invoice().date());

        assert_eq!(old_project.payed_date(),
                   new_project.payed_date());

        assert_eq!(old_project.client().title(),
                   new_project.client().title());

        assert_eq!(old_project.client().last_name(),
                   new_project.client().last_name());

        assert_eq!(old_project.client().addressing(),
                   new_project.client().addressing());

        assert_eq!(old_project.client().address(),
                   new_project.client().address());
    }
}
