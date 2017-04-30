
#[cfg(feature="document_export")]
use rustc_serialize::json::ToJson;

use clap::ArgMatches;

use asciii::BillType;
use asciii::print;
use asciii::storage;
use asciii::storage::*;
use asciii::project::{spec,Project};
use asciii::templater::Templater;
use asciii::project::spec::events::HasEvents;

use super::matches_to_search;

use ::cli::execute;
use super::path;

use std::fs;

/// Command SHOW
pub fn show(m: &ArgMatches) {
    let (search_terms, dir) = matches_to_search(m);

    let bill_type = match (m.is_present("offer"), m.is_present("invoice")) {
        (true, true)    => unreachable!("this should have been prevented by clap-rs"),
        (true, false)   => BillType::Offer,
        // (false,true) => BillType::Invoice,
        _               => BillType::Invoice, //TODO be inteligent here ( use date )
    };

    let storage = execute(||storage::setup::<Project>());

    if m.is_present("files") {
        storage.simple_with_projects(dir, Some(&search_terms), |p| {
            println!("{}: ", p.dir().display());
            for entry in fs::read_dir(p.dir()).unwrap() {
                println!("  {}", entry.unwrap().path().display())
            }
        });
    } else if let Some(detail) = m.value_of("detail") {
        show_detail(dir, &search_terms, detail);
    } else if m.is_present("empty fields"){ show_empty_fields(dir, search_terms.as_slice())
    } else if m.is_present("errors"){ show_errors(dir, search_terms.as_slice())
    } else if m.is_present("dump"){ dump_yaml(dir, search_terms.as_slice())
    } else if m.is_present("json"){ show_json(dir, search_terms.as_slice())
    } else if m.is_present("ical"){ show_ical(dir, search_terms.as_slice())
    } else if m.is_present("csv"){  show_csv( dir, search_terms.as_slice());
    } else if m.is_present("template"){ show_template(search_terms[0]);
    } else { storage.simple_with_projects(dir,
                                         Some(search_terms.as_slice()),
                                         |p|print::show_details(&p, &bill_type))
    }
}

fn show_errors(dir: StorageDir, search_terms: &[&str]) {
    let storage = execute(||storage::setup::<Project>());
    storage.simple_with_projects(dir, Some(&search_terms), |p| {
        println!("{}: ", p.short_desc());
        spec::print_specresult("offer", p.is_ready_for_offer());
        spec::print_specresult("invoice", p.is_ready_for_invoice());
        spec::print_specresult("archive", p.is_ready_for_archive());
    });
}

fn show_empty_fields(dir: StorageDir, search_terms: &[&str]) {
    let storage = execute(||storage::setup::<Project>());
    storage.simple_with_projects(dir,
                                Some(&search_terms),
                                |p| println!("{}: {}", p.short_desc(), p.empty_fields().join(", ")));
}


#[cfg(feature="document_export")]
fn show_json(dir: StorageDir, search_terms: &[&str]) {
    let storage = execute(||storage::setup::<Project>());
    storage.simple_with_projects(dir, Some(&search_terms), |p| println!("{}", p.to_json()));
}

fn show_ical(dir: StorageDir, search_terms: &[&str]) {
    let storage = execute(||storage::setup::<Project>());
    storage.simple_with_projects(dir, Some(&search_terms), |p| p.to_ical().print().unwrap());
}

fn show_detail(dir: StorageDir, search_terms: &[&str], detail: &str) {
    let storage = execute(||storage::setup::<Project>());
    storage.simple_with_projects(dir, Some(&search_terms), |p| {
        println!("{}",
                 p.get(detail).unwrap_or_else(|| String::from("Nothing found")))
    });
}

fn show_csv(dir: StorageDir, search_terms: &[&str]) {
    let storage = execute(||storage::setup::<Project>());
    storage.simple_with_projects(dir,
                                Some(&search_terms),
                                |p| println!("{}", execute(|| p.to_csv(&BillType::Invoice))));
}

#[cfg(not(feature="document_export"))]
fn show_json(_: StorageDir, _: &[&str]) {
    error!("feature temporarily disabled")
}

pub fn show_path(matches: &ArgMatches) {
    path(matches, |path| println!("{}", path.display()))
}

/// Command SHOW --template
fn show_template(name: &str) {
    let luigi = execute(storage::setup::<Project>);
    let template = execute(|| luigi.get_template_file(name));
    let templater = execute(|| Templater::from_file(&template));
    println!("{:#?}", templater.list_keywords());
}

fn dump_yaml(dir: StorageDir, search_terms: &[&str]) {
    let storage = execute(||storage::setup::<Project>());
    storage.simple_with_projects(dir, Some(&search_terms), |p| println!("{}", p.dump_yaml()));
}

