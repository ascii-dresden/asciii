//! General actions

#![allow(unused_imports)]
#![allow(dead_code)]


use chrono::prelude::*;
use bill::Currency;
use icalendar::Calendar;

use std::{env,fs};
use std::time;
use std::fmt::Write;
use std::path::{Path,PathBuf};

use util;
use super::BillType;
use storage::{self, Storage,StorageDir,Storable,StorageResult};
use project::Project;
use project::spec::IsProject;
use project::spec::IsClient;
use project::spec::Invoicable;
use project::spec::ProvidesData;
use project::spec::events::HasEvents;

pub mod error;
use self::error::*;

/// Helper method that passes projects matching the `search_terms` to the passt closure `f`
pub fn with_projects<F>(dir:StorageDir, search_terms:&[&str], f:F) -> Result<()>
    where F:Fn(&Project)->Result<()>
{
    trace!("with_projects({:?})", search_terms);
    let luigi = storage::setup::<Project>()?;
    let projects = luigi.search_projects_any(dir, search_terms)?;
    if projects.is_empty() {
        return Err(format!("Nothing found for {:?}", search_terms).into())
    }
    for project in projects {
        f(&project)?;
    }
    Ok(())
}

pub fn csv(year:i32) -> Result<String> {
    let luigi = storage::setup::<Project>()?;
    let mut projects = luigi.open_projects(StorageDir::Year(year))?;
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

    for project in projects{
        writeln!(&mut string, "{}", [
                 project.get("InvoiceNumber")                     .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("Name")                              .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("event/dates/0/begin")               .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("invoice/date")                      .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("Employees")                         .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("Responsible")                       .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("invoice/payed_date")                .unwrap_or_else(|| String::from(r#""""#)),
                 project.sum_sold().map(|c|c.value().to_string()).unwrap_or_else(|_| String::from(r#""""#)),
                 project.canceled_string().to_owned()
        ].join(splitter))?;
    }
    Ok(string)
}


/// Command DUES
pub fn open_wages() -> Result<Currency>{
    let luigi = storage::setup::<Project>()?;
    let projects = luigi.open_projects(StorageDir::Working)?;
    Ok(projects.iter()
        .filter(|p| !p.canceled() && p.age().unwrap_or(0) > 0)
        .filter_map(|p| p.wages())
        .fold(Currency::default(), |acc, x| acc + x))
}


/// Command DUES
pub fn open_payments() -> Result<Currency>{
    let luigi = storage::setup::<Project>()?;
    let projects = luigi.open_projects(StorageDir::Working)?;
    Ok(projects.iter()
       .filter(|p| !p.canceled() && !p.payed_by_client() && p.age().unwrap_or(0) > 0)
       .filter_map(|p| p.sum_sold().ok())
       .fold(Currency::default(), |acc, x| acc + x))
}


/// Testing only, tries to run complete spec on all projects.
/// TODO make this not panic :D
/// TODO move this to `spec::all_the_things`
pub fn spec() -> Result<()> {
    use project::spec::*;
    let luigi = storage::setup::<Project>()?;
    //let projects = super::execute(||luigi.open_projects(StorageDir::All));
    let projects = luigi.open_projects(StorageDir::Working)?;
    for project in projects{
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
    let luigi = storage::setup_with_git::<Project>()?;
    for project in luigi.search_projects_any(dir, search_terms)? {
        luigi.delete_project_if(&project,
                || util::really(&format!("you want me to delete {:?} [y/N]", project.dir())) && util::really("really? [y/N]")
                )?
    }
    Ok(())
}

pub fn archive_projects(search_terms:&[&str], manual_year:Option<i32>, force:bool) -> Result<Vec<PathBuf>>{
    trace!("archive_projects matching ({:?},{:?},{:?})", search_terms, manual_year,force);
    let luigi = storage::setup_with_git::<Project>()?;
    Ok( luigi.archive_projects_if(search_terms, manual_year, || force) ?)
}

pub fn archive_all_projects() -> Result<Vec<PathBuf>> {
    let luigi = storage::setup_with_git::<Project>()?;
    let mut moved_files = Vec::new();
    for project in luigi.open_projects(StorageDir::Working)?
                        .iter()
                        .filter(|p| p.is_ready_for_archive().is_ok()) {
        println!(" we could get rid of: {}", project.name().unwrap_or(""));
        moved_files.push(project.dir());
        moved_files.append(&mut luigi.archive_project(&project, project.year().unwrap())?);
    }
    Ok(moved_files)
}

/// Command UNARCHIVE <YEAR> <NAME>
/// TODO: return a list of files that have to be updated in git
pub fn unarchive_projects(year:i32, search_terms:&[&str]) -> Result<Vec<PathBuf>> {
    let luigi = storage::setup_with_git::<Project>()?;
    Ok( luigi.unarchive_projects(year, search_terms) ?)
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
    let luigi = storage::setup::<Project>()?;
    let mut cal = Calendar::new();
    if show_tasks {
        for project in luigi.open_projects(StorageDir::Working)?  {
            cal.append(&mut project.to_tasks())
        }
    }
    for project in luigi.open_projects(dir)?{
        cal.append(&mut project.to_ical())
    }
    Ok(cal.to_string())
}


