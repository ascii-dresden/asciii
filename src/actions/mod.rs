//! General actions

#![allow(unused_imports)]
#![allow(dead_code)]


use chrono::*;

use std::{env,fs};
use std::time;
use std::fmt::Write;
use std::path::{Path,PathBuf};

use util;
use super::BillType;
use storage::{Storage,StorageDir,Storable,Selection};
use project::Project;
use fill_docs::fill_template;

pub mod error;
use self::error::*;

/// Sets up an instance of `Storage`.
fn setup_luigi() -> Result<Storage<Project>> {
    trace!("setup_luigi()");
    let working   = try!(::CONFIG.get_str("dirs/working").ok_or("Faulty config: dirs/working does not contain a value"));
    let archive   = try!(::CONFIG.get_str("dirs/archive").ok_or("Faulty config: dirs/archive does not contain a value"));
    let templates = try!(::CONFIG.get_str("dirs/templates").ok_or("Faulty config: dirs/templates does not contain a value"));
    let storage   = try!(Storage::new(util::get_storage_path(), working, archive, templates));
    Ok(storage)
}

fn with_projects<F>(luigi: &Storage<Project>, dir:StorageDir, search_term:&str, cb:F) -> Result<()>
    where F:Fn(&Project)->Result<()>
{
    trace!("with_projects({})", search_term);
    // TODO make this use ProjectList
    let projects = try!(luigi.search_projects(dir, search_term));
    if !projects.is_empty(){
        for project in &projects{
            try!(cb(project));
        }
    } else {
        return Err(format!("Nothing found for {:?}", search_term).into());
    }
    Ok(())
}

pub fn csv(year:i32) -> Result<String> {
    let luigi = try!(setup_luigi());
    let mut projects = try!(luigi.open_projects(StorageDir::Year(year)));
    projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned())));
    projects_to_csv(&projects)
}

pub fn projects_to_csv(projects:&[Project]) -> Result<String>{
    let mut string = String::new();
    let splitter = "\";\"";
    try!(writeln!(&mut string, "\"{}\"", [ "Rnum", "Bezeichnung", "Datum", "Rechnungs", "Betreuer", "Verantwortlich", "Bezahlt am", "Betrag", "Canceled"].join(splitter)));
    for project in projects{
        try!(writeln!(&mut string, "\"{}\"", [
                 project.get("InvoiceNumber").unwrap_or_else(String::new),
                 project.get("Name").unwrap_or_else(String::new),
                 project.get("event/dates/0/begin").unwrap_or_else(String::new),
                 project.get("invoice/date").unwrap_or_else(String::new),
                 project.get("Caterers").unwrap_or_else(String::new),
                 project.get("Responsible").unwrap_or_else(String::new),
                 project.get("invoice/payed_date").unwrap_or_else(String::new),
                 project.get("Final").unwrap_or_else(String::new),
                 project.canceled_string().to_owned()
        ].join(splitter)));
    }
    Ok(string)
}

/// Creates the latex files within each projects directory, either for Invoice or Offer.
pub fn projects_to_tex(dir:StorageDir, search_term:&str, template_name:&str, bill_type:&Option<BillType>, dry_run:bool, force:bool) -> Result<()> {
    let luigi = try!(setup_luigi());
    //let search_term = "ese";
    //let template_name = "document";
    //let dir = StorageDir::Working ;

    let template_ext = ::CONFIG.get_str("extensions/output_template").expect("Faulty default config");
    let output_ext   = ::CONFIG.get_str("extensions/output_file").expect("Faulty default config");
    let convert_ext  = ::CONFIG.get_str("convert/output_extension").expect("Faulty default config");
    let trash_exts   = ::CONFIG.get("convert/trash_extensions") .expect("Faulty default config")
                               .as_vec().expect("Faulty default config")
                               .into_iter()
                               .map(|v|v.as_str()).collect::<Vec<_>>();

    let mut template_path = PathBuf::new();

    template_path.push(luigi.templates_dir());
    template_path.push(template_name);
    template_path.set_extension(template_ext);

    debug!("template file={:?} exists={}", template_path, template_path.exists());

    with_projects(&luigi, dir, search_term, |p| {

        let convert_tool = ::CONFIG.get_str("convert/tool");
        let output_folder = ::CONFIG.get_str("output_path").and_then(util::get_valid_path).expect("Faulty config \"output_path\"");

        let ready_for_offer = p.is_ready_for_offer();
        let ready_for_invoice = p.is_ready_for_invoice();
        let project_file = p.file();

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
            (&None,          Ok(_), Err(_))  => (Some(Offer), Some(p.dir().join(p.offer_file_name(output_ext).expect("this should have been cought by ready_for_offer()")))),
            (&Some(Invoice), _,      Ok(_))  |
            (&None,          _,      Ok(_))  => (Some(Invoice), Some(p.dir().join(p.invoice_file_name(output_ext).expect("this should have been cought by ready_for_invoice()")))),
            (&Some(Offer),   Err(e), _    )  => {error!("cannot create an offer, check out:{:#?}",e);(None,None)},
            (&Some(Invoice), _,      Err(e)) => {error!("cannot create an invoice, check out:{:#?}",e);(None,None)},
            (_,              Err(e), Err(_)) => {error!("Neither an Offer nor an Invoice can be created from this project\n please check out {:#?}", e);(None,None)}
        };

        //debug!("{:?} -> {:?}",(bill_type, p.is_ready_for_offer(), p.is_ready_for_invoice()), (dyn_bill_type, outfile_tex));

        if let (Some(outfile), Some(dyn_bill)) = (outfile_tex, dyn_bill_type) {
            let filled = try!(fill_template(p, &dyn_bill, &template_path));

            let pdffile = to_local_file(&outfile, convert_ext);
            let target = output_folder.join(&pdffile);

            // ok, so apparently we can create a tex file, so lets do it
            if !force && target.exists() && try!(file_age(&target)) < try!(file_age(&project_file)){
                // no wait, nothing has changed, so lets save ourselves the work
                info!("nothing to be done, {} is younger than {}\n       use -f if you don't agree", target.display(), project_file.display());
            } else {
                // \o/ we created a tex file

                if dry_run{
                    warn!("Dry run! This does not produce any output:\n * {}\n * {}", outfile.display(), pdffile.display());
                } else {
                    let outfileb = try!(p.write_to_file(&filled,&dyn_bill,output_ext));
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
    })
}

fn file_age(path:&Path) -> Result<time::Duration> {
    let metadata = try!(fs::metadata(path));
    let accessed = try!(metadata.accessed());
    Ok(try!(accessed.elapsed()))
}

/// Command SPEC
/// TODO make this not panic :D
/// TODO move this to `spec::all_the_things`
pub fn spec() -> Result<()> {
    use project::spec::*;
    let luigi = try!(setup_luigi());
    //let projects = super::execute(||luigi.open_projects(StorageDir::All));
    let projects = try!(luigi.open_projects(StorageDir::Working));
    for project in projects{
        info!("{}", project.dir().display());

        let yaml = project.yaml();
        client::validate(&yaml).map_err(|errors|for error in errors{
            println!("  error: {}", error);
        }).unwrap();

        client::full_name(&yaml);
        client::first_name(&yaml);
        client::title(&yaml);
        client::email(&yaml);


        hours::caterers_string(&yaml);
        invoice::number_long_str(&yaml);
        invoice::number_str(&yaml);
        offer::number(&yaml);
        project.age().map(|a|format!("{} days", a)).unwrap();
        project.date().map(|d|d.year().to_string()).unwrap();
        project.sum_sold().map(|c|util::currency_to_string(&c)).unwrap();
        project::manager(&yaml).map(|s|s.to_owned()).unwrap();
        project::name(&yaml).map(|s|s.to_owned()).unwrap();
    }

    Ok(())
}

pub fn delete_project_confirmation(selection:Selection) -> Result<()> {
    let luigi = try!(setup_luigi());
    debug!("{:?}",selection);
    with_projects(&luigi, selection.dir, selection.search, |p| {
        println!("you want me to delete [y/N]{:?}", p.dir());
        if util::really() {
            println!("commencing");
            try!(luigi.delete_project(p));
            Ok(())
        }else {
            try!(Err("nope"))
        }
    })
}
