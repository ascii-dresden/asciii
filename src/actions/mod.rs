//! General actions

#![allow(unused_imports)]
#![allow(dead_code)]


use chrono::*;
use bill::Currency;
use icalendar::Calendar;

use std::{env,fs};
use std::time;
use std::fmt::Write;
use std::path::{Path,PathBuf};

use util;
use super::BillType;
use storage::{Storage,StorageDir,Storable,StorageResult};
use project::Project;
use project::spec::IsProject;
use project::spec::IsClient;
use project::spec::Invoicable;
use project::spec::ProvidesData;
use project::spec::events::HasEvents;

#[cfg(feature="document_export")]
use fill_docs::fill_template;

pub mod error;
use self::error::*;

/// Sets up an instance of `Storage`.
pub fn setup_luigi() -> Result<Storage<Project>> {
    trace!("setup_luigi()");
    let working   = try!(::CONFIG.get_str("dirs/working").ok_or("Faulty config: dirs/working does not contain a value"));
    let archive   = try!(::CONFIG.get_str("dirs/archive").ok_or("Faulty config: dirs/archive does not contain a value"));
    let templates = try!(::CONFIG.get_str("dirs/templates").ok_or("Faulty config: dirs/templates does not contain a value"));
    let storage   = try!(Storage::new(util::get_storage_path(), working, archive, templates));
    try!(storage.health_check());
    Ok(storage)
}

/// Sets up an instance of `Storage`, with git turned on.
pub fn setup_luigi_with_git() -> Result<Storage<Project>> {
    trace!("setup_luigi()");
    let working   = try!(::CONFIG.get_str("dirs/working").ok_or("Faulty config: dirs/working does not contain a value"));
    let archive   = try!(::CONFIG.get_str("dirs/archive").ok_or("Faulty config: dirs/archive does not contain a value"));
    let templates = try!(::CONFIG.get_str("dirs/templates").ok_or("Faulty config: dirs/templates does not contain a value"));
    let storage   = try!(Storage::new_with_git(util::get_storage_path(), working, archive, templates));
    try!(storage.health_check());
    Ok(storage)
}


pub fn simple_with_projects<F>(dir:StorageDir, search_terms:&[&str], f:F)
    where F:Fn(&Project)
{
    match with_projects(dir, search_terms, |p| {f(p);Ok(())}){
        Ok(_) => {},
        Err(e) => error!("{}",e)
    }
}

/// Helper method that passes projects matching the `search_terms` to the passt closure `f`
/// TODO Really move this to `Storage`
pub fn with_projects<F>(dir:StorageDir, search_terms:&[&str], f:F) -> Result<()>
    where F:Fn(&Project)->Result<()>
{
    trace!("with_projects({:?})", search_terms);
    let luigi = try!(setup_luigi());
    let projects = try!(luigi.search_projects_any(dir, search_terms));
    if projects.is_empty() {
        return Err(format!("Nothing found for {:?}", search_terms).into())
    }
    for project in &projects{
        try!(f(project));
    }
    Ok(())
}

pub fn csv(year:i32) -> Result<String> {
    let luigi = try!(setup_luigi());
    let mut projects = try!(luigi.open_projects(StorageDir::Year(year)));
    projects.sort_by(|pa,pb| pa.index().unwrap_or_else(||"zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned())));
    projects_to_csv(&projects)
}

/// Produces a csv string from a list of `Project`s
/// TODO this still contains german terms
pub fn projects_to_csv(projects:&[Project]) -> Result<String>{
    let mut string = String::new();
    let splitter = ";";
    try!(writeln!(&mut string, "{}", [ "Rnum", "Bezeichnung", "Datum", "Rechnungsdatum", "Betreuer", "Verantwortlich", "Bezahlt am", "Betrag", "Canceled"].join(splitter)));
    for project in projects{
        try!(writeln!(&mut string, "{}", [
                 project.get("InvoiceNumber")                     .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("Name")                              .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("event/dates/0/begin")               .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("invoice/date")                      .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("Employees")                         .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("Responsible")                       .unwrap_or_else(|| String::from(r#""""#)),
                 project.get("invoice/payed_date")                .unwrap_or_else(|| String::from(r#""""#)),
                 project.sum_sold().map(|c|c.value().to_string()).unwrap_or_else(|_| String::from(r#""""#)),
                 project.canceled_string().to_owned()
        ].join(splitter)));
    }
    Ok(string)
}

/// Creates the latex files within each projects directory, either for Invoice or Offer.
#[cfg(feature="document_export")]
pub fn project_to_doc(project: &Project, template_name:&str, bill_type:&Option<BillType>, dry_run:bool, force:bool) -> Result<()> {

    // init_export_config()
    let template_ext  = ::CONFIG.get_str("extensions/output_template").expect("Faulty default config");
    let output_ext    = ::CONFIG.get_str("extensions/output_file").expect("Faulty default config");
    let convert_ext   = ::CONFIG.get_str("convert/output_extension").expect("Faulty default config");
    let convert_tool  = ::CONFIG.get_str("convert/tool");
    let output_folder = ::CONFIG.get_str("output_path").and_then(util::get_valid_path).expect("Faulty config \"output_path\"");
    let trash_exts    = ::CONFIG.get("convert/trash_extensions") .expect("Faulty default config")
                                .as_vec().expect("Faulty default config")
                                .into_iter()
                                .map(|v|v.as_str()).collect::<Vec<_>>();


    // construct_template_path(&template_name) {
    let mut template_path = PathBuf::new();
    template_path.push(util::get_storage_path());
    template_path.push(::CONFIG.get_str("dirs/templates").expect("Faulty config: dirs/templates does not contain a value"));
    template_path.push(template_name);
    template_path.set_extension(template_ext);
    // }

    // check stays here
    debug!("template file={:?} exists={}", template_path, template_path.exists());
    if !template_path.exists() {
        return Err(format!("Template not found at {}", template_path.display()).into())
    }


    // project_readyness(&project) {
    let ready_for_offer = project.is_ready_for_offer();
    let ready_for_invoice = project.is_ready_for_invoice();
    let project_file = project.file();

    // tiny little helper
    let to_local_file = |file:&Path, ext| {
        let mut _tmpfile = file.to_owned();
        _tmpfile.set_extension(ext);
        Path::new(_tmpfile.file_name().unwrap().into()).to_owned()
    };

    use BillType::*;
    let (dyn_bill_type, outfile_tex):
        (Option<BillType>, Option<PathBuf>) =
         match (bill_type, ready_for_offer, ready_for_invoice)
    {
        (&Some(Offer),   Ok(_), _     )  |
        (&None,          Ok(_), Err(_))  => (Some(Offer), Some(project.dir().join(project.offer_file_name(output_ext).expect("this should have been cought by ready_for_offer()")))),
        (&Some(Invoice), _,      Ok(_))  |
        (&None,          _,      Ok(_))  => (Some(Invoice), Some(project.dir().join(project.invoice_file_name(output_ext).expect("this should have been cought by ready_for_invoice()")))),
        (&Some(Offer),   Err(e), _    )  => {error!("cannot create an offer, check out:{}",e);(None,None)},
        (&Some(Invoice), _,      Err(e)) => {error!("cannot create an invoice, check out:{}",e);(None,None)},
        (_,              Err(e), Err(_)) => {error!("Neither an Offer nor an Invoice can be created from this project\n please check out {}", e);(None,None)}
    };

    // }

    //debug!("{:?} -> {:?}",(bill_type, project.is_ready_for_offer(), project.is_ready_for_invoice()), (dyn_bill_type, outfile_tex));

    if let (Some(outfile), Some(dyn_bill)) = (outfile_tex, dyn_bill_type) {
        let filled = try!(fill_template(project, &dyn_bill, &template_path));

        let pdffile = to_local_file(&outfile, convert_ext);
        let target = output_folder.join(&pdffile);

        // ok, so apparently we can create a tex file, so lets do it
        if !force && target.exists() && try!(file_age(&target)) < try!(file_age(&project_file)) {
            // no wait, nothing has changed, so lets save ourselves the work
            info!("nothing to be done, {} is younger than {}
                         use --force if you don't agree
                         use --pdf to only rebuild the pdf",
                  target.display(),
                  project_file.display());
            unimplemented!();
        } else {
            // \o/ we created a tex file

            if dry_run{
                warn!("Dry run! This does not produce any output:\n * {}\n * {}", outfile.display(), pdffile.display());
            } else {
                let outfileb = try!(project.write_to_file(&filled,&dyn_bill,output_ext));
                debug!("{} vs\n        {}", outfile.display(), outfileb.display());
                util::pass_to_command(&convert_tool, &[&outfileb]);
            }
            // clean up expected trash files
            for trash_ext in trash_exts.iter().filter_map(|x|*x){
                let trash_file = to_local_file(&outfile, trash_ext);
                if  trash_file.exists() {
                    try!(fs::remove_file(&trash_file));
                    debug!("just deleted: {}", trash_file.display())
                }
                else {
                    debug!("I expected there to be a {}, but there wasn't any ?", trash_file.display())
                }
            }
            if pdffile.exists(){
                debug!("now there is be a {:?} -> {:?}", pdffile, target);
                try!(fs::rename(&pdffile, &target));
            }
        }
    }

    Ok(())
}

/// Creates the latex files within each projects directory, either for Invoice or Offer.
#[cfg(feature="document_export")]
pub fn projects_to_doc(dir:StorageDir, search_term:&str, template_name:&str, bill_type:&Option<BillType>, dry_run:bool, force:bool) -> Result<()> {
    with_projects(dir, &[search_term], |p| project_to_doc(p, template_name, bill_type, dry_run, force) )
}

fn file_age(path:&Path) -> Result<time::Duration> {
    let metadata = try!(fs::metadata(path));
    let accessed = try!(metadata.accessed());
    Ok(try!(accessed.elapsed()))
}

/// Command DUES
pub fn open_wages() -> Result<Currency>{
    let luigi = try!(setup_luigi());
    let projects = try!(luigi.open_projects(StorageDir::Working));
    Ok(projects.iter()
        .filter(|p| !p.canceled() && p.age().unwrap_or(0) > 0)
        .filter_map(|p| p.wages())
        .fold(Currency::default(), |acc, x| acc + x))
}


/// Command DUES
pub fn open_payments() -> Result<Currency>{
    let luigi = try!(setup_luigi());
    let projects = try!(luigi.open_projects(StorageDir::Working));
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
    let luigi = try!(setup_luigi());
    //let projects = super::execute(||luigi.open_projects(StorageDir::All));
    let projects = try!(luigi.open_projects(StorageDir::Working));
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
    let luigi = try!(setup_luigi_with_git());
    for project in try!(luigi.search_projects_any(dir, search_terms)) {
        try!(luigi.delete_project_if(&project,
                || util::really(&format!("you want me to delete {:?} [y/N]", project.dir())) && util::really("really? [y/N]")
                ))
    }
    Ok(())
}

pub fn archive_projects(search_terms:&[&str], manual_year:Option<i32>, force:bool) -> Result<Vec<PathBuf>>{
    trace!("archive_projects matching ({:?},{:?},{:?})", search_terms, manual_year,force);
    let luigi = try!(setup_luigi_with_git());
    Ok(try!( luigi.archive_projects_if(search_terms, manual_year, || force) ))
}

pub fn archive_all_projects() -> Result<Vec<PathBuf>> {
    let luigi = try!(setup_luigi_with_git());
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
    let luigi = try!(setup_luigi_with_git());
    Ok(try!( luigi.unarchive_projects(year, search_terms) ))
}

/// Command CALENDAR
pub fn calendar(dir: StorageDir) -> Result<String> {
    let luigi = try!(setup_luigi());
    let projects = try!(luigi.open_projects(dir));
    let mut cal = Calendar::new();
    for project in projects {
        cal.append(&mut project.to_ical())
    }
    Ok(cal.to_string())
}


