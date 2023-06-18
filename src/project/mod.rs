//! Project file parsing and evaluation.
//!
//! This module implements all functionality of a project.

use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf}
};

use anyhow::{bail, Error};
use chrono::{prelude::*, Duration};
use maplit::hashmap;
use tempdir::TempDir;
use yaml_rust::Yaml;

use bill::BillItem;
use icalendar::{Calendar, CalendarDateTime, Component, Todo};
use semver::Version;

use crate::{
    storage::{list_path_content, repo::GitStatus, Storable, StorableAndTempDir, StorageError},
    templater::{IsKeyword, Templater},
    util::{get_valid_path, yaml}
};

pub mod product;
pub mod spec;
mod spec_yaml;
mod yaml_provider;

mod computed_field;
pub mod error;

#[cfg(test)]
mod tests;

#[cfg(feature = "serialization")]
pub mod export;
#[cfg(feature = "deserialization")]
pub mod import;
#[cfg(feature = "serialization")]
use self::export::*;

use self::{
    error::ProjectError,
    product::{Product, ProductError},
    spec::{HasEmployees, Invoicable, IsClient, IsProject, Offerable, Redeemable, Validatable},
    yaml_provider::*
};

pub use self::computed_field::ComputedField;

/// Represents a Project.
///
/// A project is storable, contains products, and you can create an offer or invoice from it.
/// The main implementation is done in [`spec`](spec/index.html).
#[derive(Clone)]
pub struct Project {
    file_path: PathBuf,
    git_status: Option<GitStatus>,
    file_content: String,
    yaml: Yaml
}

impl Project {
    /// Access to inner data
    pub fn yaml(&self) -> &Yaml {
        &self.yaml
    }

    /// Opens a project from file path;
    pub fn open<S: AsRef<OsStr> + std::fmt::Debug + ?Sized>(pathish: &S) -> Result<Project, Error> {
        log::trace!("Project::open({:?});", pathish);
        let file_path = Path::new(&pathish);
        let file_content = fs::read_to_string(file_path)?;
        let project = Project {
            file_path: file_path.to_owned(),
            git_status: None,
            yaml: yaml::parse(&file_content).unwrap_or_else(|e| {
                log::error!("syntax error in {}\n  {}", file_path.display(), e);
                Yaml::Null
            }),
            file_content
        };

        let validation = project
            .validate()
            .and(project.client().validate())
            .and(project.invoice().validate())
            .and(project.offer().validate())
            .and(project.hours().validate())
            .and(<dyn Redeemable>::validate(&project));

        if !validation.validation_errors.is_empty() {
            let name = project.short_desc();
            log::warn!("project {:?}:", name);
            for err in validation.validation_errors {
                println!(" * {}", err);
            }
        }
        Ok(project)
    }

    /// import from yaml file
    #[cfg(feature = "deserialization")]
    pub fn parse_yaml(&self) -> Result<import::Project, Error> {
        import::from_str(&self.file_content)
    }

    /// (feature deactivated) import from yaml file
    #[cfg(not(feature = "deserialization"))]
    pub fn parse_yaml(&self) -> Result<(), Error> {
        bail!(error::ProjectError::FeatureDeactivated)
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

    #[cfg(feature = "serialization")]
    /// export to JSON
    pub fn to_json(&self) -> Result<String, Error> {
        let complete: Complete = self.export();
        Ok(serde_json::to_string(&complete)?)
    }

    #[cfg(not(feature = "serialization"))]
    /// feature deactivateda) export to JSON
    pub fn to_json(&self) -> Result<String, Error> {
        bail!(error::ProjectError::FeatureDeactivated)
    }

    /// Used mostly for testing purposes
    pub fn from_file_content(content: &str) -> Result<Project, Error> {
        Ok(Project {
            file_path: PathBuf::new(),
            git_status: None,
            yaml: yaml::parse(content).unwrap(),
            file_content: String::from(content)
        })
    }

    /// wrapper around `yaml::get()` with replacement
    pub fn field(&self, path: &str) -> Option<String> {
        ComputedField::from(path)
            .get(self)
            .or_else(|| yaml::get_to_string(self.yaml(), path))
    }

    /// Returns the struct `Client`, which abstracts away client specific stuff.
    pub fn client(&self) -> Client<'_> {
        Client { inner: self }
    }

    /// Returns the struct `Offer`, which abstracts away offer specific stuff.
    pub fn offer(&self) -> Offer<'_> {
        Offer { inner: self }
    }

    /// Returns the struct `Invoice`, which abstracts away invoice specific stuff.
    pub fn invoice(&self) -> Invoice<'_> {
        Invoice { inner: self }
    }

    /// Returns the struct `Invoice`, which abstracts away invoice specific stuff.
    pub fn hours(&self) -> Hours<'_> {
        Hours { inner: self }
    }

    /// Ready to produce offer.
    ///
    /// Ready to send an **offer** to the client.
    ///
    /// Returns list of missing fields, empty vector if ready.
    pub fn is_missing_for_offer(&self) -> Vec<String> {
        self.offer()
            .validate()
            .and(self.client().validate())
            .and(self.validate())
            .missing_fields
    }

    /// Valid to produce invoice
    ///
    /// Ready to send an **invoice** to the client.
    ///
    /// Returns list of missing fields, empty vector if ready.
    pub fn is_missing_for_invoice(&self) -> Vec<String> {
        let mut missing = self.is_missing_for_offer();
        missing.extend(self.invoice().validate().missing_fields);
        missing
    }

    /// Completely done and in the past.
    ///
    /// Ready to be **h:
    ///
    /// Returns list of missing fields, empty vector if ready.
    pub fn is_ready_for_archive(&self) -> Vec<String> {
        if self.canceled() {
            Vec::new()
        } else {
            <dyn Redeemable>::validate(self)
                .and(self.hours().validate())
                .missing_fields
        }
    }

    pub fn to_csv(&self, bill_type: BillType) -> Result<String, Error> {
        use std::fmt::Write;
        let (offer, invoice) = self.bills()?;
        let bill = match bill_type {
            BillType::Offer => offer,
            BillType::Invoice => invoice
        };
        let mut csv_string = String::new();
        let splitter = ";";

        writeln!(
            &mut csv_string,
            "{}",
            ["#", "Bezeichnung", "Menge", "EP", "Steuer", "Preis"].join(splitter)
        )?;

        for items in bill.items_by_tax.values() {
            for (index, item) in items.iter().enumerate() {
                write!(&mut csv_string, "{};", &index.to_string())?;
                write!(&mut csv_string, "{};", item.product.name)?;
                write!(&mut csv_string, "{};", item.amount)?;
                write!(&mut csv_string, "{:.2};", item.product.price.as_float())?;
                write!(&mut csv_string, "{:.2};", item.product.tax)?;
                writeln!(&mut csv_string, "{:.2}", (item.product.price * item.amount).as_float())?;
            }
        }
        Ok(csv_string)
    }

    pub fn debug(&self) -> Debug {
        self.into()
    }

    /// Check Templated for replaceable markers
    pub fn empty_fields(&self) -> Vec<String> {
        self.file_content.list_keywords()
    }

    /// Fill certain field
    pub fn replace_field(&self, field: &str, value: &str) -> Result<(), Error> {
        // fills the template
        let filled = Templater::new(&self.file_content)
            .fill_in_field(field, value)
            .finalize()
            .filled;

        match yaml::parse(&filled) {
            Ok(_) => {
                let mut file = File::create(self.file())?;
                file.write_all(filled.as_bytes())?;
                file.sync_all()?;
                Ok(())
            },
            Err(e) => {
                log::error!(
                    "The resulting document is no valid yaml. SORRY!\n{}\n\n{}",
                    filled
                        .lines()
                        .enumerate()
                        .map(|(n, l)| format!("{:>3}. {}\n", n, l))
                        .collect::<String>(), //line numbers :D
                    e
                );
                bail!(e)
            }
        }
    }

    /// Time between event and creation of invoice
    pub fn our_bad(&self) -> Option<Duration> {
        let event = self.event_date().ok()?;
        let invoice = self.invoice().date().ok().unwrap_or_else(Utc::today);
        let diff = invoice.signed_duration_since(event);
        if diff > Duration::zero() {
            Some(diff)
        } else {
            None
        }
    }

    /// Time between creation of invoice and payment
    pub fn their_bad(&self) -> Option<Duration> {
        let invoice = self.invoice().date().ok().unwrap_or_else(Utc::today);
        let payed = self.payed_date().ok().unwrap_or_else(Utc::today);
        Some(invoice.signed_duration_since(payed))
    }

    /// What I need to do
    ///
    /// Produces an iCal calendar from this project.
    pub fn to_tasks(&self) -> Calendar {
        //return if self.canceled();
        let mut cal = Calendar::new();

        let event = self.event_date().ok();
        let invoice = self.invoice().date().ok();
        let payed = self.payed_date().ok();
        let wages = self.hours().wages_date().ok();
        let today = Utc::today();

        let days_since = |date: Date<Utc>| (today.signed_duration_since(date)).num_days();

        if let Some(event) = event {
            match (invoice, payed, wages) {
                // we need to issue an invoice invoice
                (None, None, _) if today >= event => {
                    cal.push(Self::task_issue_invoice(event));
                },
                (None, None, _) if today < event => { /* no need to worry yet */ },

                // they haven't payed us yet
                (Some(invoice), None, _) if days_since(invoice) >= 14 => {
                    cal.push(self.task_follow_up(invoice));
                },
                (Some(invoice), None, _) if days_since(invoice) < 14 => { /* they have 14 days before we complain */ },

                // we need to pay the employees
                (Some(_), Some(payed), None) => {
                    cal.push(self.task_pay_employees(payed));
                },

                // everything's all set to close this
                (Some(_), Some(_), Some(wages)) if days_since(wages) > 7 => {
                    cal.push(self.task_close_project(wages));
                },
                _ => log::warn!(
                    "{}",
                    lformat!(
                        "weird task edgecase in {:?}:\n{:?}",
                        self.file(),
                        (event, invoice, payed, wages)
                    )
                )
            }
        }
        cal
    }

    fn task_issue_invoice(event_date: Date<Utc>) -> Todo {
        Todo::new()
            .summary(&lformat!("Create an Invoice"))
            .due(CalendarDateTime::from(
                (event_date + Duration::days(14)).and_hms(11, 10, 0)
            ))
            .priority(6)
            .done()
    }

    fn task_pay_employees(&self, payed_date: Date<Utc>) -> Todo {
        let days_since_payed = (Utc::today().signed_duration_since(payed_date)).num_days();
        Todo::new()
            .summary(&lformat!(
                "{}: Hungry employees!",
                self.invoice().number_str().unwrap_or_default()
            ))
            .description(&lformat!(
                "Pay {}\nYou have had the money for {} days!",
                self.hours().employees_string().unwrap_or_default(),
                days_since_payed
            ))
            .due(CalendarDateTime::from(
                (payed_date + Duration::days(14)).and_hms(11, 10, 0)
            ))
            .done()
    }

    fn task_follow_up(&self, invoice_date: Date<Utc>) -> Todo {
        let days_since_invoice = (Utc::today().signed_duration_since(invoice_date)).num_days();
        let mut follow_up = Todo::new();
        follow_up.summary(&lformat!("Inquire about: \"{event}\"!", event = self.name().unwrap()));
        follow_up.description(&lformat!("{inum }{event:?} on {invoice_date} ({days} days ago) was already invoiced but is still not marked as payed.\nPlease check for incoming payments! You can ask {client} ({mail}).",
                                       event = self.name().unwrap(),
                                       days = days_since_invoice,
                                       inum = self.invoice().number_str().unwrap_or_default(),
                                       invoice_date = invoice_date.format("%d.%m.%Y").to_string(),
                                       client = self.client().full_name().unwrap_or_default(),
                                       mail = self.client().email().unwrap_or(""),
                                       ));
        follow_up.priority(3);
        if days_since_invoice > 14 {
            follow_up.summary(&lformat!(
                "{rnum}: payment is {weeks} weeks late: \"{event}\"",
                rnum = self.invoice().number_str().unwrap_or_default(),
                weeks = days_since_invoice / 7,
                event = self.name().unwrap()
            ));
            follow_up.priority(10);
        }
        follow_up
    }

    fn task_close_project(&self, wages_date: Date<Utc>) -> Todo {
        let days_since_wages = (Utc::today().signed_duration_since(wages_date)).num_days();
        Todo::new()
            .summary(&lformat!("Archive {}", self.name().unwrap()))
            .description(&lformat!(
                "{:?} has been finished for {} days, get rid of it!",
                self.name().unwrap(),
                days_since_wages
            ))
            .done()
    }

    fn item_from_desc_and_value<'y>(
        &self,
        desc: &'y Yaml,
        values: &'y Yaml
    ) -> Result<(BillItem<Product<'y>>, BillItem<Product<'y>>), Error> {
        let get_f64 = |yaml, path| {
            self.get_direct(yaml, path)
                .and_then(|y| y.as_f64().or_else(|| y.as_i64().map(|y| y as f64)))
        };

        let product = Product::from_desc_and_value(desc, values, self.tax().ok())?;

        let offered = get_f64(values, "amount").ok_or_else(|| ProductError::MissingAmount(product.name.to_owned()))?;

        let sold = get_f64(values, "sold");
        // TODO: test this
        let sold = if let Some(returned) = get_f64(values, "returned") {
            // if "returned", there must be no "sold"
            if sold.is_some() {
                bail!(ProductError::AmbiguousAmounts(product.name.to_owned()));
            }
            if returned > offered {
                bail!(ProductError::TooMuchReturned(product.name.to_owned()));
            }
            offered - returned
        } else if let Some(sold) = sold {
            sold
        } else {
            offered
        };

        Ok((
            BillItem {
                amount: offered,
                product
            },
            BillItem { amount: sold, product }
        ))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BillType {
    Offer,
    Invoice
}

impl ToString for BillType {
    fn to_string(&self) -> String {
        match *self {
            BillType::Offer => "Offer",
            BillType::Invoice => "Invoice"
        }
        .to_owned()
    }
}

/// Functionality to create output files
pub trait Exportable {
    /// Where to export to
    fn export_dir(&self) -> PathBuf;

    /// Filename of the offer output file.
    fn offer_file_name(&self, extension: &str) -> Option<String>;

    /// Filename of the invoice output file. **Careful!** uses today's date.
    fn invoice_file_name(&self, extension: &str) -> Option<String>;

    fn output_file_exists(&self, bill_type: BillType) -> bool {
        match bill_type {
            BillType::Offer => self.offer_file_exists(),
            BillType::Invoice => self.invoice_file_exists()
        }
    }

    fn output_file(&self, bill_type: BillType) -> Option<PathBuf> {
        match bill_type {
            BillType::Offer => self.offer_file(),
            BillType::Invoice => self.invoice_file()
        }
    }

    fn offer_file(&self) -> Option<PathBuf> {
        let output_folder = get_valid_path(crate::CONFIG.get_str("output_path"));
        let convert_ext = crate::CONFIG.get_str("document_export/output_extension");
        match (output_folder, self.offer_file_name(convert_ext)) {
            (Some(folder), Some(name)) => folder.join(name).into(),
            _ => None
        }
    }

    fn invoice_file(&self) -> Option<PathBuf> {
        let output_folder = get_valid_path(crate::CONFIG.get_str("output_path"));
        let convert_ext = crate::CONFIG.get_str("document_export/output_extension");
        match (output_folder, self.invoice_file_name(convert_ext)) {
            (Some(folder), Some(name)) => folder.join(name).into(),
            _ => None
        }
    }

    fn offer_file_exists(&self) -> bool {
        self.offer_file().map_or(false, |f| f.exists())
    }

    fn invoice_file_exists(&self) -> bool {
        self.invoice_file().map_or(false, |f| f.exists())
    }

    fn write_to_path<P: AsRef<OsStr> + fmt::Debug>(content: &str, target: &P) -> Result<(), Error> {
        log::trace!("writing content ({}bytes) to {:?}", content.len(), target);
        let mut file = File::create(Path::new(target))?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    fn full_file_path(&self, bill_type: BillType, ext: &str) -> Result<PathBuf, Error> {
        match bill_type {
            BillType::Offer => self.full_offer_file_path(ext),
            BillType::Invoice => self.full_invoice_file_path(ext)
        }
    }

    fn full_offer_file_path(&self, ext: &str) -> Result<PathBuf, Error> {
        if let Some(target) = self.offer_file_name(ext) {
            Ok(self.export_dir().join(target))
        } else {
            bail!(ProjectError::CantDetermineTargetFile)
        }
    }

    fn full_invoice_file_path(&self, ext: &str) -> Result<PathBuf, Error> {
        if let Some(target) = self.invoice_file_name(ext) {
            Ok(self.export_dir().join(target))
        } else {
            bail!(ProjectError::CantDetermineTargetFile)
        }
    }

    fn write_to_file(&self, content: &str, bill_type: BillType, ext: &str) -> Result<PathBuf, Error> {
        match bill_type {
            BillType::Offer => self.write_to_offer_file(content, ext),
            BillType::Invoice => self.write_to_invoice_file(content, ext)
        }
    }

    fn write_to_offer_file(&self, content: &str, ext: &str) -> Result<PathBuf, Error> {
        let full_path = self.full_offer_file_path(ext)?;
        Self::write_to_path(content, &full_path)?;
        Ok(full_path)
    }

    fn write_to_invoice_file(&self, content: &str, ext: &str) -> Result<PathBuf, Error> {
        let full_path = self.full_invoice_file_path(ext)?;
        Self::write_to_path(content, &full_path)?;
        Ok(full_path)
    }
}

impl Exportable for Project {
    fn export_dir(&self) -> PathBuf {
        Storable::dir(self)
    }

    fn offer_file_name(&self, extension: &str) -> Option<String> {
        let num = self.offer().number().ok()?;
        let name = slug::slugify(IsProject::name(self).ok()?);
        Some(format!("{} {}.{}", num, name, extension))
    }

    fn invoice_file_name(&self, extension: &str) -> Option<String> {
        let num = self.invoice().number_str()?;
        let name = slug::slugify(self.name().ok()?);
        //let date = Local::today().format("%Y-%m-%d").to_string();
        let date = self.invoice().date().ok()?.format("%Y-%m-%d").to_string();
        Some(format!("{} {} {}.{}", num, name, date, extension))
    }
}

impl Storable for Project {
    fn file_extension() -> String {
        crate::CONFIG.get_to_string("extensions.project_file")
    }

    fn from_template(
        project_name: &str,
        template: &Path,
        fill: &HashMap<&str, String>
    ) -> Result<StorableAndTempDir<Self>, Error> {
        let template_name = template.file_stem().unwrap().to_str().unwrap();

        let event_date = (Utc::today() + Duration::days(14)).format("%d.%m.%Y").to_string();
        let created_date = Utc::today().format("%d.%m.%Y").to_string();

        // fill template with these values
        let default_fill = hashmap! {
            "TEMPLATE"      => template_name.to_owned(),
            "PROJECT-NAME"  => project_name.to_owned(),
            "DATE-EVENT"    => event_date,
            "DATE-CREATED"  => created_date,
            "TAX"           => crate::CONFIG.get_to_string("defaults/tax"),
            "SALARY"        => crate::CONFIG.get_to_string("defaults/salary"),
            "MANAGER"       => crate::CONFIG.get_str_or("user/name").unwrap_or("").to_string(),
            "TIME-START"    => String::new(),
            "TIME-END"      => String::new(),
            "VERSION"       => crate::VERSION.to_string(),
        };

        // fills the template
        let file_content = Templater::from_file(template)?
            .fill_in_data(fill)
            .fix()
            .fill_in_data(&default_fill)
            .finalize()
            .filled;

        log::debug!("remaining template fields: {:#?}", file_content.list_keywords());

        // generates a temp file
        let temp_dir = TempDir::new(project_name).unwrap();
        let temp_file = temp_dir
            .path()
            .join(slug::slugify(project_name) + "." + &Self::file_extension());

        // write into a file
        let mut file = File::create(&temp_file)?;
        file.write_all(file_content.as_bytes())?;
        file.sync_all()?;

        let yaml = match yaml::parse(&file_content) {
            Ok(y) => y,
            Err(e) => {
                log::error!(
                    "The created document is no valid yaml. SORRY!\n{}\n\n{}",
                    file_content
                        .lines()
                        .enumerate()
                        .map(|(n, l)| format!("{:>3}. {}\n", n, l))
                        .collect::<String>(), //line numbers :D
                    e
                );
                bail!(e)
            }
        };

        // project now lives in the temp_file
        let project = Project {
            file_path: temp_file,
            git_status: None,
            file_content,
            yaml
        };

        Ok(StorableAndTempDir {
            storable: project,
            temp_dir
        })
    }

    fn prefix(&self) -> Option<String> {
        self.invoice().number_str()
    }

    fn index(&self) -> Option<String> {
        let prefix = self.invoice().number_long_str().unwrap_or_else(|| String::from("zzzz"));
        match (self.invoice().date().ok(), self.modified_date()) {
            (Some(date), _) | (None, Some(date)) => Some(format!("{0}{1}", prefix, date.format("%Y%m%d"))),
            (None, None) => None
        }
    }

    fn short_desc(&self) -> String {
        self.name().ok().map(ToOwned::to_owned).unwrap_or_else(|| {
            format!(
                "unnamed: {:?}",
                self.dir().file_name().expect("the end was \"..\", but why?")
            )
        })
    }

    fn modified_date(&self) -> Option<Date<Utc>> {
        self.get_dmy("event.dates.0.begin")
            .or_else(|_| self.get_dmy("created"))
            .or_else(|_| self.get_dmy_legacy_range("date"))
            .ok()
    }

    fn file(&self) -> PathBuf {
        self.file_path.clone()
    } // TODO: reconsider returning PathBuf at all
    fn set_file(&mut self, new_file: &Path) {
        self.file_path = new_file.to_owned();
    }

    fn set_git_status(&mut self, status: GitStatus) {
        self.git_status = Some(status);
    }

    /// Ask a project for its gitstatus
    #[cfg(feature = "git_statuses")]
    fn get_git_status(&self) -> GitStatus {
        self.git_status.as_ref().map_or(GitStatus::Unknown, ToOwned::to_owned)
    }

    /// Opens a yaml and parses it.
    fn open_folder(folder_path: &Path) -> Result<Project, Error> {
        let project_file_extension = crate::CONFIG.get_to_string("extensions.project_file");
        let file_path = list_path_content(folder_path)?
            .iter()
            .find(|f| f.extension().unwrap_or_else(|| OsStr::new("")) == project_file_extension.as_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| StorageError::NoProjectFile(folder_path.to_owned()))?;
        Self::open_file(&file_path)
    }

    fn open_file(file_path: &Path) -> Result<Project, Error> {
        Project::open(file_path)
    }

    /// Checks against a certain key-val pair.
    fn matches_filter(&self, key: &str, val: &str) -> bool {
        self.field(key)
            .map_or(false, |c| c.to_lowercase().contains(&val.to_lowercase()))
    }

    /// UNIMPLEMENTED: Checks against a certain search term.
    ///
    /// TODO: compare agains [`InvoiceNumber`], [`ClientFullName`], Email, event/name, invoice/official Etc
    fn matches_search(&self, term: &str) -> bool {
        let search = term.to_lowercase();
        self.invoice()
            .number_str()
            .map_or(false, |num| num.to_lowercase().contains(&search))
            || Storable::short_desc(self).to_lowercase().contains(&search)
    }

    fn is_ready_for_archive(&self) -> bool {
        Project::is_ready_for_archive(self).is_empty()
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
    pub file_path: PathBuf,
    //temp_dir: Option<PathBuf>, // TODO
    pub git_status: Option<GitStatus>,
    pub yaml: Yaml
}

impl<'a> From<&'a Project> for Debug {
    fn from(project: &'a Project) -> Debug {
        Debug {
            file_path: project.file_path.clone(),
            git_status: project.git_status.clone(),
            yaml: project.yaml.clone()
        }
    }
}

impl fmt::Debug for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "{:?}", self.debug())
        write!(f, "{:?}{:?}", self.name(), self.file())
    }
}
