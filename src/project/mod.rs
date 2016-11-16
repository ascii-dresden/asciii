//! Project file parsing and evaluation.
//!
//! This module implements all functionality of a project.

use std::fs::File;
use std::io::prelude::*;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::error::Error as ErrorTrait;
use std::collections::HashMap;

use chrono::*;
use yaml_rust::Yaml;
use tempdir::TempDir;
use slug;
use bill::Currency;
//use semver::Version;

use super::BillType;
use util;
use util::yaml;
use storage::list_path_content;
use storage::{Storable,StorageResult};
use storage::ErrorKind as StorageErrorKind;
use storage::repo::GitStatus;
use templater::{Templater, IsKeyword};

#[export_macro]
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

#[cfg(feature="document_export")]
mod tojson;

//#[cfg(test)] mod tests;

use self::spec::ProvidesData;
use self::spec::{IsProject, IsClient};
use self::spec::{Offerable, Invoicable, Redeemable, Validatable};
use self::spec::events::HasEvents;
use self::error::{ErrorKind, ErrorList, SpecResult, Result};

pub use self::computed_field::ComputedField;





static PROJECT_FILE_EXTENSION:&'static str = "yml";

/// Output of `Project::debug()`.
///
/// A project is storable, contains products, and you can create an offer or invoice from it.
#[derive(Debug)]
pub struct DebugProject {
    file_path: PathBuf,
    //temp_dir: Option<PathBuf>, // TODO
    git_status: Option<GitStatus>,
    yaml: Yaml
}

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
    pub fn client<'a>(&'a self) -> Client<'a> {
        Client { inner: self }
    }

    /// Returns the struct `Offer`, which abstracts away offer specific stuff.
    pub fn offer<'a>(&'a self) -> Offer<'a> {
        Offer { inner: self }
    }

    /// Returns the struct `Invoice`, which abstracts away invoice specific stuff.
    pub fn invoice<'a>(&'a self) -> Invoice<'a> {
        Invoice { inner: self }
    }

    pub fn invoice_num(&self) -> Option<String>{
        self.invoice().number_str()
    }

    pub fn payed_by_client(&self) -> bool{
        self.payed_date().is_some()
    }

    pub fn payed_caterers(&self) -> bool{
        self.wages_date().is_some()
    }

    pub fn caterers(&self) -> String {
        self.caterers_string().unwrap_or_else(String::new)
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

    fn write_to_path<P:AsRef<OsStr> + Debug>(content:&str, target:&P) -> Result<PathBuf> {
        trace!("writing content ({}bytes) to {:?}", content.len(), target);
        let mut file = try!(File::create(Path::new(target)));
        try!(file.write_all(content.as_bytes()));
        try!(file.sync_all());
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
        } else {Err(ErrorKind::CantDetermineTargetFile.into())}
    }

    fn write_to_invoice_file(&self,content:&str, ext:&str) -> Result<PathBuf> {
        if let Some(target) = self.invoice_file_name(ext){
            Self::write_to_path(content, &self.dir().join(&target))
        } else {Err(ErrorKind::CantDetermineTargetFile.into())}
    }


    /// Ready to produce offer.
    ///
    /// Ready to send an **offer** to the client.
    pub fn is_ready_for_offer(&self) -> SpecResult{
        let client             = self.client();
        let client_validation  = client.validate();

        let offer              = self.offer();
        let offer_validation   = offer.validate();

        let project_validation = self.validate();
        offer_validation.and(client_validation).and(project_validation)
    }

    /// Valid to produce invoice
    ///
    /// Ready to send an **invoice** to the client.
    pub fn is_ready_for_invoice(&self) -> SpecResult{
        let invoice_validation = self.invoice().validate();

        self.is_ready_for_offer().and(invoice_validation)
    }

    /// Completely done and in the past.
    ///
    /// Ready to be **archived**.
    pub fn is_ready_for_archive(&self) -> SpecResult {
        if self.canceled(){
            Ok(())
        } else {
            Redeemable::validate(self)
        }
    }

    /// TODO move to `IsProjectExt`
    pub fn age(&self) -> Option<i64> {
        self.modified_date().map(|date| (Local::today() - date).num_days() )
    }

    pub fn to_csv(&self, bill_type:&BillType) -> Result<String>{
        use std::fmt::Write;
        let (offer, invoice) = try!(self.bills());
        let bill = match *bill_type{ BillType::Offer => offer, BillType::Invoice => invoice };
        let mut csv_string = String::new();
        let splitter = ";";

        try!(writeln!(&mut csv_string, "{}", [ "#", "Bezeichnung", "Menge", "EP", "Steuer", "Preis"].join(splitter)));


        for items in bill.items_by_tax.values(){
            for (index,item) in items.iter().enumerate(){
                                        try!(write!(&mut csv_string, "{};",  &index.to_string()));
                                        try!(write!(&mut csv_string, "{};",  item.product.name));
                                        try!(write!(&mut csv_string, "{};",  item.amount.to_string()));
                                        try!(write!(&mut csv_string, "{:.2};",  item.product.price.as_float()));
                                        try!(write!(&mut csv_string, "{:.2};",  item.product.tax));
                                        try!(write!(&mut csv_string, "{:.2}\n", (item.product.price * item.amount).as_float()));
            }
        }
        Ok(csv_string)
    }

    pub fn wages(&self) -> Option<Currency> {
        if let (Some(total), Some(salary)) = (self.total(), self.salary()){
            Some(total * salary)
        } else{None}
    }

    pub fn sum_sold(&self) -> Result<Currency> {
        let (_,invoice) = try!(self.bills());
        Ok(invoice.net_total())
    }

    pub fn debug(&self) -> DebugProject{
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
                let mut file = try!(File::create(self.file()));
                try!(file.write_all(filled.as_bytes()));
                try!(file.sync_all());
                Ok(())
            },
            Err(e) => {
                error!("The resulting document is no valid yaml. SORRY!\n{}\n\n{}",
                       filled.lines().enumerate().map(|(n,l)| format!("{:>3}. {}\n",n,l)).collect::<String>(), //line numbers :D
                       e.description());
                Err(e.into())
            }
        }
    }

    pub fn our_bad(&self) -> Option<Duration> {
        let event   = try_some!(self.event_date());
        let invoice = self.invoice().date().unwrap_or_else(UTC::today);
        let diff = invoice - event;
        if diff > Duration::zero() {
            Some(diff)
        } else {
            None
        }
    }

    pub fn their_bad(&self) -> Option<Duration> {
        let invoice = self.invoice().date().unwrap_or_else(UTC::today);
        let payed   = self.payed_date().unwrap_or_else(UTC::today);
        Some(invoice - payed)
    }
}

impl ProvidesData for Project {
    fn data(&self) -> &Yaml{
        self.yaml()
    }
}

impl IsProject for Project { }
impl Redeemable for Project { }

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

impl HasEvents for Project {
}

/// This is returned by [Product::client()](struct.Project.html#method.client).
pub struct Client<'a> {
    inner: &'a Project
}

impl<'a> ProvidesData for Client<'a> {
    fn data(&self) -> &Yaml{
        self.inner.data()
    }
}

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

/// This is returned by [Product::offer()](struct.Project.html#method.offer).
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


/// This is returned by [Product::invoice()](struct.Project.html#method.invoice).
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

// TODO move to spec_traint.rs
impl<'a> IsClient for Client<'a> { }

impl Storable for Project{
    fn file_extension() -> &'static str {PROJECT_FILE_EXTENSION}
    fn from_template(project_name:&str,template:&Path, fill: &HashMap<&str,String>) -> StorageResult<Project> {
        let template_name = template.file_stem().unwrap().to_str().unwrap();

        let event_date = (Local::today() + Duration::days(14)).format("%d.%m.%Y").to_string();
        let created_date = Local::today().format("%d.%m.%Y").to_string();

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
        let filled = try!(Templater::from_file(template))
            .fill_in_data(&fill).fix()
            .fill_in_data(&default_fill)
            .finalize()
            .filled;

        debug!("remaining template fields: {:#?}", filled.list_keywords());

        // generates a temp file
        let temp_dir  = TempDir::new(project_name).unwrap();
        let temp_file = temp_dir.path().join(slug::slugify(project_name) + "." + Self::file_extension());

        // write into a file
        let mut file = try!( File::create(&temp_file) );
        try!(file.write_all(filled.as_bytes()));
        try!(file.sync_all());

        let yaml = match yaml::parse(&filled){
            Ok(y) => y,
            Err(e) => {
                error!("The created document is no valid yaml. SORRY!\n{}\n\n{}",
                       filled.lines().enumerate().map(|(n,l)| format!("{:>3}. {}\n",n,l)).collect::<String>(), //line numbers :D
                       e.description());
                return Err(e.into())
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
        self.invoice_num()
    }

    fn index(&self) -> Option<String>{
        if let Some(date) = self.modified_date() {
            self.invoice().number_str()
                .map(|num| format!("{1}{0}", date.format("%Y%m%d").to_string(),num))
                .or_else(||Some(date.format("zzz%Y%m%d").to_string()))
        } else {
            None
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
                     .and_then(|s|
                               util::yaml::parse_dmy_date_range(s))
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
    fn open(folder_path:&Path) -> StorageResult<Project>{
        let file_path = try!(try!(list_path_content(folder_path)).iter()
            .filter(|f|f.extension().unwrap_or(&OsStr::new("")) == PROJECT_FILE_EXTENSION)
            .nth(0).map(|b|b.to_owned())
            .ok_or(StorageErrorKind::ProjectDoesNotExist));
        Self::open_file(&file_path)
    }

    fn open_file(file_path:&Path) -> StorageResult<Project>{
        let file_content = try!(File::open(&file_path)
                                .and_then(|mut file| {
                                    let mut content = String::new();
                                    file.read_to_string(&mut content).map(|_| content)
                                }));
        Ok(Project{
            file_path: file_path.to_owned(),
            _temp_dir: None,
            git_status: None,
            yaml: try!(yaml::parse(&file_content)),
            file_content: file_content,
        })
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
        self.invoice_num().map_or(false, |num|num.to_lowercase().contains(&search))
        ||
        Storable::short_desc(self).to_lowercase().contains(&search)
    }

    fn is_ready_for_archive(&self) -> bool {
        Project::is_ready_for_archive(self).is_ok()
    }
}

impl<'a> From<&'a Project> for DebugProject{
    fn from(project: &'a Project) -> DebugProject{
        DebugProject{
            file_path:  project.file_path.clone(),
            git_status: project.git_status.clone(),
            yaml:       project.yaml.clone()
        }
    }
}


#[cfg(test)]
mod test {
    use std::path::Path;
    use ::project::spec;
    use ::project::Project;
    use ::storage::Storable;

    #[test]
    fn compare_basics(){
        println!("{:?}", ::std::env::current_dir());
        let new_project = Project::open_file(Path::new("./tests/current.yml")).unwrap();
        let old_project = Project::open_file(Path::new("./tests/old.yml")).unwrap();
        let new_yaml = new_project.yaml();
        let old_yaml = old_project.yaml();
        let config = &::CONFIG;

        assert_eq!(spec::project::name(&old_yaml), spec::project::name(&new_yaml));

        //assert_eq!(spec::project::storage(&old_yaml), //fails
        //           spec::project::storage(&new_yaml));

        assert_eq!(spec::offer::number(&old_yaml), spec::offer::number(&new_yaml));

        //assert_eq!(spec::date::offer(&old_yaml), //fails
        //           spec::date::offer(&new_yaml));

        assert_eq!(spec::invoice::number_str(&old_yaml), spec::invoice::number_str(&new_yaml));
        assert_eq!(spec::date::invoice(&old_yaml), spec::date::invoice(&new_yaml));
        assert_eq!(spec::date::payed(&old_yaml), spec::date::payed(&new_yaml));
        assert_eq!(spec::client::title(&old_yaml), spec::client::title(&new_yaml));
        assert_eq!(spec::client::last_name(&old_yaml), spec::client::last_name(&new_yaml));
        assert_eq!(spec::client::addressing(&old_yaml), spec::client::addressing(&new_yaml));
    }
}
