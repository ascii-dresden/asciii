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
use tempdir::TempDir;
use slug;

use bill::{Bill, Currency, Tax};
use icalendar::*;
//use semver::Version;

use super::BillType;
use util;
use util::yaml;
use storage::list_path_content;
use storage::{Storable,StorageResult};
use storage::ErrorKind as StorageErrorKind;
use storage::repo::GitStatus;
use templater::{Templater, IsKeyword};

//#[export_macro]
macro_rules! try_some {
    ($expr:expr) => (match $expr {
        Some(val) => val,
        None => return None,
    });
}

pub mod product;
pub mod spec;

pub mod error;
mod computed_field;

#[cfg(feature="json")]
pub mod tojson;

//#[cfg(test)] mod tests;

use self::spec::ProvidesData;
use self::spec::{IsProject, IsClient};
use self::spec::{Offerable, Invoicable, Redeemable, Validatable, HasEmployees};
use self::spec::events::HasEvents;
use self::error::{ErrorKind, ErrorList, SpecResult, Result};
use self::product::Product;
use self::product::error as product_error;


pub use self::computed_field::ComputedField;





static PROJECT_FILE_EXTENSION:&'static str = "yml";

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

/// Represents a Project.
///
/// A project is storable, contains products, and you can create an offer or invoice from it.
/// The main implementation is done in [`spec`](spec/index.html).
//#[derive(Clone)]
pub struct Project {
    file_path: PathBuf,
    _temp_dir: Option<TempDir>,
    git_status: Option<GitStatus>,
    file_content: String,
    yaml: Yaml
}

pub mod export {
    use super::spec::*;
    use super::Project;

    pub trait Exportable<T> {
        fn export(self) -> T;
    }

    fn opt_str(opt:Option<&str>) -> Option<String> {
        opt.map(|e|e.to_owned())
    }

    #[derive(Debug, PartialEq)]
    #[cfg_attr(feature = "serialization", derive(Serialize,Deserialize))]
    pub struct Client {
        title: Option<String>,
        first_name: Option<String>,
        last_name: Option<String>,
        full_name: Option<String>,
        address: Option<String>,
        email: Option<String>,
        addressing : Option<String>,
    }

    impl Exportable<Client> for Project {
        fn export(self) -> Client {
            Client {
                full_name:   self.client().full_name(),
                addressing:  self.client().addressing(),
                email:       opt_str(self.client().email()),
                last_name:   opt_str(self.client().last_name()),
                first_name:  opt_str(self.client().first_name()),
                title:       opt_str(self.client().title()),
                address:     opt_str(self.client().address()),
            }
        }
    }

    //#[derive(Serialize, Debug)]
    //pub struct Bills<'a> {
    //    offer: Bill<Product<'a>>,
    //    invoice: Bill<Product<'a>>,
    //}

    //impl<'a> Exportable<Bills<'a>> for Project {
    //    fn export(self) -> Bills<'a> {
    //        let (offer, invoice) = self.bills().unwrap();
    //        Bills {
    //            offer, invoice
    //        }
    //    }
    //}


}

impl Project {
    /// Access to inner data
    pub fn yaml(&self) -> &Yaml{ &self.yaml }

    pub fn open<S: AsRef<OsStr> + ?Sized>(pathish: &S) -> Result<Project> {
        let file_path = Path::new(&pathish);
        let file_content = File::open(&file_path)
                                .and_then(|mut file| {
                                    let mut content = String::new();
                                    file.read_to_string(&mut content).map(|_| content)
                                })?;
        Ok(Project{
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

    /// wrapper around yaml::get() with replacement
    pub fn get(&self, path:&str) -> Option<String> {
        ComputedField::from(path).get(self).or_else(||
            yaml::get_to_string(self.yaml(),path)
        )
    }

    /// either `"canceled"` or `""`
    pub fn canceled_string(&self) -> &'static str{
        if self.canceled(){"canceled"}
        else {""}
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

    pub fn payed_by_client(&self) -> bool{
        self.payed_date().is_some()
    }

    //#[deprecated]
    pub fn employees_string(&self) -> String {
        self.hours().employees_string().unwrap_or_else(String::new)
    }

    #[deprecated]
    pub fn caterers(&self) -> String {
        self.employees_string()
    }

    /// Filename of the offer output file.
    pub fn offer_file_name(&self, extension:&str) -> Option<String>{
        let num = try_some!(self.offer().number());
        let name = slug::slugify(try_some!(IsProject::name(self)));
        Some(format!("{} {}.{}",num,name,extension))
    }

    /// Filename of the invoice output file. **Carefull!** uses today's date.
    pub fn invoice_file_name(&self, extension:&str) -> Option<String>{
        let num = try_some!(self.invoice().number_str());
        let name = slug::slugify(try_some!(self.name()));
        //let date = Local::today().format("%Y-%m-%d").to_string();
        let date = try_some!(self.invoice().date()).format("%Y-%m-%d").to_string();
        Some(format!("{} {} {}.{}",num,name,date,extension))
    }

    pub fn output_file_exists(&self, bill_type:&BillType) -> bool {
        match *bill_type{
            BillType::Offer   => self.offer_file_exists(),
            BillType::Invoice => self.invoice_file_exists()
        }
    }

    pub fn output_file(&self, bill_type:&BillType) -> Option<PathBuf> {
        match *bill_type{
            BillType::Offer   => self.offer_file(),
            BillType::Invoice => self.invoice_file()
        }
    }

    pub fn offer_file(&self) -> Option<PathBuf> {
        let output_folder = ::CONFIG.get_str("output_path").and_then(util::get_valid_path);
        let convert_ext  = ::CONFIG.get_str("convert/output_extension").expect("Faulty default config");
        match (output_folder, self.offer_file_name(convert_ext)) {
            (Some(folder), Some(name)) => folder.join(&name).into(),
            _ => None
        }
    }

    pub fn invoice_file(&self) -> Option<PathBuf>{
        let output_folder = ::CONFIG.get_str("output_path").and_then(util::get_valid_path);
        let convert_ext  = ::CONFIG.get_str("convert/output_extension").expect("Faulty default config");
        match (output_folder, self.invoice_file_name(convert_ext)) {
            (Some(folder), Some(name)) => folder.join(&name).into(),
            _ => None
        }
    }

    pub fn offer_file_exists(&self) -> bool {
        self.offer_file().map(|f|f.exists()).unwrap_or(false)
    }

    pub fn invoice_file_exists(&self) -> bool {
        self.invoice_file().map(|f|f.exists()).unwrap_or(false)
    }

    fn write_to_path<P:AsRef<OsStr> + fmt::Debug>(content:&str, target:&P) -> Result<PathBuf> {
        trace!("writing content ({}bytes) to {:?}", content.len(), target);
        let mut file = File::create(Path::new(target))?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        Ok(Path::new(target).to_owned())
    }

    pub fn write_to_file(&self,content:&str, bill_type:&BillType,ext:&str) -> Result<PathBuf> {
        match *bill_type{
            BillType::Offer   => self.write_to_offer_file(content, ext),
            BillType::Invoice => self.write_to_invoice_file(content, ext)
        }
    }

    fn write_to_offer_file(&self,content:&str, ext:&str) -> Result<PathBuf> {
        if let Some(target) = self.offer_file_name(ext){
            Self::write_to_path(content, &self.dir().join(&target))
        } else {bail!(ErrorKind::CantDetermineTargetFile)}
    }

    fn write_to_invoice_file(&self,content:&str, ext:&str) -> Result<PathBuf> {
        if let Some(target) = self.invoice_file_name(ext){
            Self::write_to_path(content, &self.dir().join(&target))
        } else {bail!(ErrorKind::CantDetermineTargetFile)}
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

    /// TODO move to `IsProjectExt`
    pub fn age(&self) -> Option<i64> {
        self.modified_date().map(|date| (UTC::today().signed_duration_since(date)).num_days() )
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

    pub fn wages(&self) -> Option<Currency> {
        if let (Some(total), Some(salary)) = (self.hours().total(), self.hours().salary()){
            Some(total * salary)
        } else{None}
    }

    pub fn sum_sold(&self) -> Result<Currency> {
        let (_,invoice) = self.bills()?;
        Ok(invoice.net_total())
    }

    pub fn debug(&self) -> Debug {
        self.into()
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

    pub fn empty_fields(&self) -> Vec<String>{
        self.file_content.list_keywords()
    }

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

    /// Time Since Event
    pub fn our_bad(&self) -> Option<Duration> {
        let event   = try_some!(self.event_date());
        let invoice = self.invoice().date().unwrap_or_else(UTC::today);
        let diff = invoice.signed_duration_since(event);
        if diff > Duration::zero() {
            Some(diff)
        } else {
            None
        }
    }

    pub fn their_bad(&self) -> Option<Duration> {
        let invoice = self.invoice().date().unwrap_or_else(UTC::today);
        let payed   = self.payed_date().unwrap_or_else(UTC::today);
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
        let today   = UTC::today();

        let days_since = |date:Date<UTC>| (today.signed_duration_since(date)).num_days();

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

    fn task_issue_invoice(&self, event_date: Date<UTC>) -> Todo {
        Todo::new().summary(&lformat!("Create an Invoice"))
                   .due((event_date + Duration::days(14)).and_hms(11, 10, 0))
                   .priority(6)
                   .done()
    }

    fn task_pay_employees(&self, payed_date: Date<UTC>) -> Todo {
        let days_since_payed = (UTC::today().signed_duration_since(payed_date)).num_days();
        Todo::new().summary(&lformat!("{}: Hungry employees!", self.invoice().number_str().unwrap_or_else(String::new)))
            .description( &lformat!("Pay {}\nYou have had the money for {} days!",
                                   self.employees_string(),
                                   days_since_payed))
            .due((payed_date + Duration::days(14)).and_hms(11, 10, 0))
            .done()
    }

    fn task_follow_up(&self, invoice_date: Date<UTC>) -> Todo {
        let days_since_invoice = (UTC::today().signed_duration_since(invoice_date)).num_days();
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

    fn task_close_project(&self, wages_date: Date<UTC>) -> Todo {
            let days_since_wages = (UTC::today().signed_duration_since(wages_date)).num_days();
            Todo::new().summary( &lformat!("Archive {}", self.name().unwrap()))
                       .description( &lformat!("{:?} has been finished for {} days, get rid of it!",
                                              self.name().unwrap(),
                                              days_since_wages))
                       .done()
    }

}

impl ProvidesData for Project {
    fn data(&self) -> &Yaml{
        self.yaml()
    }
}

impl IsProject for Project {
    fn long_desc(&self) -> String {
        use std::fmt::Write;
        let mut out_string = String::new();

        if let Some(responsible) = self.responsible() {
            out_string += &lformat!("Responsible: {}", responsible);
        }

        if let Some(employees) = self.hours().employees_string() {
            writeln!(out_string, "\n{}", employees).unwrap();
        }

        out_string
    }
}

impl Redeemable for Project {
    fn bills(&self) -> product::Result<(Bill<Product>, Bill<Product>)> {
        let mut offer: Bill<Product> = Bill::new();
        let mut invoice: Bill<Product> = Bill::new();

        let service = || Product {
            name: "Service",
            unit: Some("h"),
            tax: Tax::new(0.0), // TODO this ought to be in the config
            price: self.hours().salary().unwrap_or(Currency::new())
        };

        if let Some(total) = self.hours().total() {
            if total.is_normal() {
                offer.add_item(total, service());
                invoice.add_item(total, service());
            }
        }

        let raw_products =
            self.get_hash("products")
                .ok_or_else(||product_error::Error::from(product_error::ErrorKind::UnknownFormat))?;

        // let document_tax =  // TODO activate this once the tax no longer 19%

        for (desc,values) in raw_products {
            let (offer_item, invoice_item) = self.item_from_desc_and_value(desc, values)?;
            if offer_item.amount.is_normal()   { offer.add(offer_item); }
            if invoice_item.amount.is_normal() { invoice.add(invoice_item); }
        }

        Ok((offer,invoice))
    }
}

impl Validatable for Project {
    fn validate(&self) -> SpecResult {
        let mut errors = ErrorList::new();
        if self.name().is_none(){errors.push("name")}
        if self.event_date().is_none(){errors.push("date")}
        if self.responsible().is_none(){errors.push("manager")}
        if self.format().is_none(){errors.push("format")}
        //if hours::salary().is_none(){errors.push("salary")}

        if errors.is_empty(){ Ok(()) }
        else { Err(errors) }
    }
}

impl Storable for Project {
    fn file_extension() -> &'static str {PROJECT_FILE_EXTENSION}
    fn from_template(project_name:&str,template:&Path, fill: &HashMap<&str,String>) -> StorageResult<Project> {
        let template_name = template.file_stem().unwrap().to_str().unwrap();

        let event_date = (UTC::today() + Duration::days(14)).format("%d.%m.%Y").to_string();
        let created_date = UTC::today().format("%d.%m.%Y").to_string();

        // fill template with these values
        let default_fill = hashmap!{
            "TEMPLATE"      => template_name.to_owned(),
            "PROJECT-NAME"  => project_name.to_owned(),
            "DATE-EVENT"    => event_date,
            "DATE-CREATED"  => created_date,
            "TAX"           => ::CONFIG.get_to_string("defaults/tax")
                                       .expect("Faulty config: field defaults/tax does not contain a value"),
            "SALARY"        => ::CONFIG.get_to_string("defaults/salary")
                                       .expect("Faulty config: field defaults/salary does not contain a value"),
            "MANAGER"       => ::CONFIG.get_str("user/name").unwrap_or("").to_string(),
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
        let temp_file = temp_dir.path().join(slug::slugify(project_name) + "." + Self::file_extension());

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

    fn modified_date(&self) -> Option<Date<UTC>> {
        self.get_dmy( "event.dates.0.begin")
            .or_else(||self.get_dmy("created"))
            .or_else(||self.get_dmy("date"))
            // probably the dd-dd.mm.yyyy format
            .or_else(||self.get_str("date")
                           .and_then(|s| util::yaml::parse_dmy_date_range(s))
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
    fn open_folder(folder_path:&Path) -> StorageResult<Project>{
        let file_path = list_path_content(folder_path)?.iter()
            .filter(|f|f.extension().unwrap_or(&OsStr::new("")) == PROJECT_FILE_EXTENSION)
            .nth(0).map(|b|b.to_owned())
            .ok_or(StorageErrorKind::ProjectDoesNotExist)?;
        Self::open_file(&file_path)
    }

    fn open_file(file_path:&Path) -> StorageResult<Project>{
        Ok(Project::open(file_path)?)
    }

    /// Checks against a certain key-val pair.
    fn matches_filter(&self, key: &str, val: &str) -> bool{
        self.get(key).map_or(false, |c| c.to_lowercase().contains(&val.to_lowercase()))
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

impl<'a> From<&'a Project> for Debug {
    fn from(project: &'a Project) -> Debug {
        Debug {
            file_path:  project.file_path.clone(),
            git_status: project.git_status.clone(),
            yaml:       project.yaml.clone()
        }
    }
}

impl HasEvents for Project { }

impl fmt::Debug for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "{:?}", self.debug())
        write!(f, "{:?}{:?}", self.name(), self.file())
    }
}





/// This is returned by `[Product::client()](struct.Project.html#method.client)`.
pub struct Client<'a> {
    inner: &'a Project
}

impl<'a> ProvidesData for Client<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> IsClient for Client<'a> { }

impl<'a> Validatable for Client<'a> {
    fn validate(&self) -> SpecResult {
        let mut errors = self.field_exists( &[
                                             //"client/email", // TODO make this a requirement
                                             "client/address",
                                             "client/title",
                                             "client/last_name",
                                             "client/first_name"
                                             ]);


        if self.addressing().is_none() {
            errors.push("client_addressing");
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}





/// This is returned by [`Product::offer()`](struct.Project.html#method.offer).
pub struct Offer<'a> {
    inner: &'a Project
}

impl<'a> ProvidesData for Offer<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> Offerable for Offer<'a> { }

impl<'a> Validatable for Offer<'a> {
    fn validate(&self) -> SpecResult {
        //if IsProject::canceled(self) {
        //    return Err(vec!["canceled"]);
        //}

        let mut errors = self.field_exists(&["offer.date", "offer.appendix", "manager"]);
        if Offerable::date(self).is_none() {
            errors.push("offer_date_format");
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())

    }
}





/// This is returned by [`Product::invoice()`](struct.Project.html#method.invoice).
pub struct Invoice<'a> {
    inner: &'a Project
}

impl<'a> ProvidesData for Invoice<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> Invoicable for Invoice<'a> { }

impl<'a> Validatable for Invoice<'a> {
    fn validate(&self) -> SpecResult {
        let mut errors = self.field_exists(&["invoice.number"]);

        // if super::offer::validate(yaml).is_err() {errors.push("offer")}
        if self.date().is_none() {
            errors.push("invoice_date");
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}





/// This is returned by [`Product::hours()`](struct.Project.html#method.hours).
pub struct Hours<'a> {
    inner: &'a Project
}

impl<'a> ProvidesData for Hours<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

impl<'a> HasEmployees for Hours<'a> { }

impl<'a> Validatable for Hours<'a> {
    fn validate(&self) -> SpecResult {
        let mut errors = ErrorList::new();
        if !self.employees_payed() { errors.push("employees_payed"); }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
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
