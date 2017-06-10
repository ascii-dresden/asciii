
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

use super::{matches_to_search, matches_to_selection};

use ::cli::execute;
use super::path;

use std::fs;

/// Command SHOW
pub fn show(m: &ArgMatches) {
    let (search_terms, _) = matches_to_search(m);
    let selection = matches_to_selection(m);

    let bill_type = match (m.is_present("offer"), m.is_present("invoice")) {
        (true, true)    => unreachable!("this should have been prevented by clap-rs"),
        (true, false)   => BillType::Offer,
        // (false,true) => BillType::Invoice,
        _               => BillType::Invoice, //TODO be inteligent here ( use date )
    };

    let storage = execute(||storage::setup::<Project>());

    if m.is_present("files") {
        storage.with_selection(&selection, |p| {
            println!("{}: ", p.dir().display());
            for entry in fs::read_dir(p.dir()).unwrap() {
                println!("  {}", entry.unwrap().path().display())
            }
        }).unwrap();
    } else if let Some(detail) = m.value_of("detail") { show_detail(&selection, detail);
    } else if m.is_present("empty fields"){ show_empty_fields(&selection)
    } else if m.is_present("errors"){ show_errors(&selection)
    } else if m.is_present("dump"){ dump_yaml(&selection)
    } else if m.is_present("json"){ show_json(&selection)
    } else if m.is_present("ical"){ show_ical(&selection)
    } else if m.is_present("csv"){  show_csv(&selection);
    } else if m.is_present("template"){ show_template(search_terms[0]);
    } else { storage.with_selection(&selection, |p| print::show_details(&p, &bill_type)).unwrap();
    }
}

fn show_errors(selection: &StorageSelection) {
    let storage = execute(||storage::setup::<Project>());
    storage.with_selection(selection, |p| {
        println!("{}: ", p.short_desc());
        spec::print_specresult("offer", p.is_ready_for_offer());
        spec::print_specresult("invoice", p.is_ready_for_invoice());
        spec::print_specresult("archive", p.is_ready_for_archive());
    }).unwrap();
}

fn show_empty_fields(selection: &StorageSelection) {
    let storage = execute(||storage::setup::<Project>());
    storage.with_selection(selection, |p| println!("{}: {}", p.short_desc(), p.empty_fields().join(", "))).unwrap();
}


#[cfg(feature="document_export")]
fn show_json(selection: &StorageSelection) {
    let storage = execute(||storage::setup::<Project>());
    storage.with_selection(selection, |p| println!("{}", p.to_json())).unwrap();
}

fn show_ical(selection: &StorageSelection) {
    let storage = execute(||storage::setup::<Project>());
    storage.with_selection(selection, |p| p.to_ical().print().unwrap()).unwrap();
}

fn show_detail(selection: &StorageSelection, detail: &str) {
    let storage = execute(||storage::setup::<Project>());
    storage.with_selection(selection, |p| {
        println!("{}",
                 p.get(detail).unwrap_or_else(|| String::from("Nothing found")))
    }).unwrap();
}

fn show_csv(selection: &StorageSelection) {
    let storage = execute(||storage::setup::<Project>());
    storage.with_selection(selection, |p| println!("{}", execute(|| p.to_csv(&BillType::Invoice)))).unwrap();
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

fn dump_yaml(selection: &StorageSelection) {
    let storage = execute(||storage::setup::<Project>());
    storage.with_selection(selection, |p| println!("{}", p.dump_yaml())).unwrap();
}

