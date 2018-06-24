//! General actions


use chrono::prelude::*;
use bill::Currency;
use icalendar::Calendar;
#[cfg(feature = "meta")]
use toml;

use std::fmt::Write;
#[cfg(feature = "meta")] use std::fs;

use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;

use util;
use storage::{self, StorageDir, Storable};
use project::Project;
use project::spec::*;

pub mod error;
use self::error::*;

/// Helper method that passes projects matching the `search_terms` to the passt closure `f`
pub fn with_projects<F>(dir:StorageDir, search_terms:&[&str], f:F) -> Result<()>
    where F:Fn(&Project)->Result<()>
{
    trace!("with_projects({:?})", search_terms);
    let projects = storage::setup::<Project>()?.search_projects_any(dir, search_terms)?;
    if projects.is_empty() {
        return Err(format!("Nothing found for {:?}", search_terms).into())
    }
    for project in projects {
        f(&project)?;
    }
    Ok(())
}

pub fn csv(year:i32) -> Result<String> {
    let mut projects = storage::setup::<Project>()?.open_projects(StorageDir::Year(year))?;
    projects.sort_by(|pa,pb| pa.index().unwrap_or_else(||"zzzz".to_owned()).cmp( &pb.index().unwrap_or_else(||"zzzz".to_owned())));
    projects_to_csv(&projects)
}

/// Produces a csv string from a list of `Project`s
pub fn projects_to_csv(projects:&[Project]) -> Result<String>{
    let mut string = String::new();
    let splitter = ";";

    writeln!(&mut string, "{}",
             [
             lformat!("INum"), // Rnum
             lformat!("Designation"), //Bezeichnung
             lformat!("Date"), // Datum
             lformat!("InvoiceDate"), // Rechnungsdatum
             lformat!("Caterer"), // Betreuer
             lformat!("Responsible"), //Verantwortlich
             lformat!("Payed on"), // Bezahlt am
             lformat!("Amount"), // Betrag
             lformat!("Canceled") //Canceled
             ]
             .join(splitter))?;

    for project in projects {
        writeln!(&mut string, "{}", [
                 project.field("InvoiceNumber")                     .unwrap_or_else(|| String::from(r#""""#)),
                 project.field("Name")                              .unwrap_or_else(|| String::from(r#""""#)),
                 project.field("event/dates/0/begin")               .unwrap_or_else(|| String::from(r#""""#)),
                 project.field("invoice/date")                      .unwrap_or_else(|| String::from(r#""""#)),
                 project.field("Employees")                         .unwrap_or_else(|| String::from(r#""""#)),
                 project.field("Responsible")                       .unwrap_or_else(|| String::from(r#""""#)),
                 project.field("invoice/payed_date")                .unwrap_or_else(|| String::from(r#""""#)),
                 project.sum_sold().map(|c|c.value().to_string()).unwrap_or_else(|_| String::from(r#""""#)),
                 String::from(if project.canceled(){"canceled"} else {""})
        ].join(splitter))?;
    }
    Ok(string)
}


fn open_payments(projects: &[Project]) -> Currency {
   projects.iter()
           .filter(|&p| !p.canceled() && !p.is_payed() && p.age().unwrap_or(0) > 0)
           .filter_map(|p| p.sum_sold().ok())
           .fold(Currency::default(), |acc, x| acc + x)
}

fn open_wages(projects: &[Project]) -> Currency {
    projects.iter()
            .filter(|p| !p.canceled() && p.age().unwrap_or(0) > 0)
            .filter_map(|p| p.hours().net_wages())
            .fold(Currency::default(), |acc, x| acc + x)
}

fn unpayed_employees(projects: &[Project]) -> HashMap<String, Currency> {
    let mut buckets = HashMap::new();
    let employees = projects.iter()
                            .filter(|p| !p.canceled() && p.age().unwrap_or(0) > 0)
                            .filter_map(|p| p.hours().employees())
                            .flat_map(|e| e.into_iter());

    for employee in employees {
        let bucket = buckets.entry(employee.name.clone()).or_insert_with(Currency::new);
        *bucket = *bucket + employee.salary;
    }
    buckets
}

#[derive(Debug)]
pub struct Dues {
    pub acc_sum_sold: Currency,
    pub acc_wages: Currency,
    pub unpayed_employees: HashMap<String, Currency>,
}

/// Command DUES
pub fn dues() -> Result<Dues> {
    let projects = storage::setup::<Project>()?.open_projects(StorageDir::Working)?;
    let acc_sum_sold: Currency = open_payments(&projects);
    let acc_wages = open_wages(&projects);
    let unpayed_employees = unpayed_employees(&projects);

    Ok(Dues{ acc_sum_sold, acc_wages, unpayed_employees})
}

/// Testing only, tries to run complete spec on all projects.
/// TODO make this not panic :D
/// TODO move this to `spec::all_the_things`
pub fn spec() -> Result<()> {
    use project::spec::*;
    let projects = storage::setup::<Project>()?.open_projects(StorageDir::Working)?;
    //let projects = super::execute(||storage.open_projects(StorageDir::All));
    for project in projects {
        info!("{}", project.dir().display());

        project.client().validate().map_err(|errors| println!("{}", errors)).unwrap();

        project.client().full_name();
        project.client().first_name();
        project.client().title();
        project.client().email();


        project.hours().employees_string();
        project.invoice().number_long_str();
        project.invoice().number_str();
        project.offer().number();
        project.age().map(|a|format!("{} days", a)).unwrap();
        project.modified_date().map(|d|d.year().to_string()).unwrap();
        project.sum_sold().map(|c|util::currency_to_string(&c)).unwrap();
        project.responsible().map(|s|s.to_owned()).unwrap();
        project.name().map(|s|s.to_owned()).unwrap();
    }

    Ok(())
}

pub fn delete_project_confirmation(dir: StorageDir, search_terms:&[&str]) -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    for project in storage.search_projects_any(dir, search_terms)? {
        storage.delete_project_if(&project,
                || util::really(&lformat!("really?"))
                )?
    }
    Ok(())
}

pub fn archive_projects(search_terms:&[&str], manual_year:Option<i32>, force:bool) -> Result<Vec<PathBuf>>{
    trace!("archive_projects matching ({:?},{:?},{:?})", search_terms, manual_year,force);
    Ok( storage::setup_with_git::<Project>()?.archive_projects_if(search_terms, manual_year, || force) ?)
}

pub fn archive_all_projects() -> Result<Vec<PathBuf>> {
    let storage = storage::setup_with_git::<Project>()?;
    let mut moved_files = Vec::new();
    for project in storage.open_projects(StorageDir::Working)?
                        .iter()
                        .filter(|p| p.is_ready_for_archive().is_ok()) {
        info!("{}", lformat!("we could get rid of: {}", project.name().unwrap_or("")));
        moved_files.push(project.dir());
        moved_files.append(&mut storage.archive_project(&project, project.year().unwrap())?);
    }
    Ok(moved_files)
}

/// Command UNARCHIVE <YEAR> <NAME>
/// TODO: return a list of files that have to be updated in git
pub fn unarchive_projects(year:i32, search_terms:&[&str]) -> Result<Vec<PathBuf>> {
    Ok( storage::setup_with_git::<Project>()?.unarchive_projects(year, search_terms) ?)
}

/// Produces a calendar from the selected `StorageDir`
pub fn calendar(dir: StorageDir) -> Result<String> {
    calendar_with_tasks(dir, true)
}

/// Command CALENDAR
///
/// Produces a calendar including tasks from the selected `StorageDir`
pub fn calendar_and_tasks(dir: StorageDir) -> Result<String> {
    calendar_with_tasks(dir, false)
}

pub fn calendar_with_tasks(dir: StorageDir, show_tasks:bool) -> Result<String> {
    let storage = storage::setup::<Project>()?;
    let mut cal = Calendar::new();
    if show_tasks {
        for project in storage.open_projects(StorageDir::Working)?  {
            cal.append(&mut project.to_tasks())
        }
    }
    for project in storage.open_projects(dir)?{
        cal.append(&mut project.to_ical())
    }
    Ok(cal.to_string())
}

/// Clone the repo
///
pub fn clone_remote(url: &str, to: &str) -> Result<()> {
    trace!("cloning {:?} to {:?}", url, to);
    Command::new("git")
        .args(&["clone", url, to])
        .status()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    Ok(())
}

/// Shared extra information stored in the repo
#[cfg_attr(feature = "meta", derive(Serialize))]
#[cfg_attr(feature = "meta", derive(Deserialize))]
#[derive(Debug)]
pub struct MetaStore {
    pub api: ApiKeys
}

#[cfg_attr(feature = "meta", derive(Serialize))]
#[cfg_attr(feature = "meta", derive(Deserialize))]
#[derive(Debug)]
/// ApiKeys store
pub struct ApiKeys {
    pub keys: Vec<String>,
    pub users: HashMap<String, String>
}


/// Parses meta store
#[cfg(feature = "meta")]
pub fn parse_meta() -> Result<MetaStore> {
    let path = storage::setup::<Project>()?.get_extra_file("meta.toml")?;
    let file_content = fs::read_to_string(&path)?;
    let store: MetaStore = toml::from_str(&file_content)?;

    Ok(store)
}

/// get ApiKeys for server
#[cfg(feature = "meta")]
pub fn get_api_keys() -> Result<ApiKeys> {
    Ok(parse_meta()?.api)
}

pub fn store_meta() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.get_repository()?;
    let path = storage.get_extra_file("meta.toml")?;
    if repo.add(&[path]).success() {
        Ok(())
    } else {
        bail!(ErrorKind::AddingFailed)
    }
}
