
#[cfg(feature="document_export")]
use rustc_serialize::json::ToJson;

use clap::ArgMatches;

use asciii::{BillType, print};
use asciii::storage::*;
//use asciii::storage::error::*;
use asciii::project::{spec, Project};
use asciii::templater::Templater;
use asciii::project::spec::events::HasEvents;

use super::{matches_to_search, matches_to_selection};

use ::cli::error::*;
use super::path;

use std::fs;

/// Command SHOW
pub fn show(m: &ArgMatches) -> Result<()>{
    let (search_terms, _) = matches_to_search(m);
    let selection = matches_to_selection(m);

    let bill_type = match (m.is_present("offer"), m.is_present("invoice")) {
        (true, true)    => unreachable!("this should have been prevented by clap-rs"),
        (true, false)   => BillType::Offer,
        // (false,true) => BillType::Invoice,
        _               => BillType::Invoice, //TODO be inteligent here ( use date )
    };

    if m.is_present("files") { show_files(&selection)
    } else if let Some(detail) = m.value_of("detail") { show_detail(&selection, detail)
    } else if m.is_present("empty fields"){ show_empty_fields(&selection)
    } else if m.is_present("errors"){ show_errors(&selection)
    } else if m.is_present("dump"){ dump_yaml(&selection)
    } else if m.is_present("json"){ show_json(&selection)
    } else if m.is_present("ical"){ show_ical(&selection)
    } else if m.is_present("csv"){  show_csv(&selection)
    } else if m.is_present("template"){ show_template(search_terms[0])
    } else { Ok(setup::<Project>()?
                .with_selection(&selection, |p| print::show_details(&p, &bill_type))?)
    }
}

fn show_files(selection: &StorageSelection) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(&selection, |p| {
        println!("{}: ", p.dir().display());
        for entry in fs::read_dir(p.dir()).unwrap() {
            println!("  {}", entry.unwrap().path().display())
        }
    })?)
}

fn show_errors(selection: &StorageSelection) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(selection, |p| {
        println!("{}: ", p.short_desc());
        spec::print_specresult("offer", p.is_ready_for_offer());
        spec::print_specresult("invoice", p.is_ready_for_invoice());
        spec::print_specresult("archive", p.is_ready_for_archive());
    })?)
}

fn show_empty_fields(selection: &StorageSelection) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(selection, |p| println!("{}: {}", p.short_desc(), p.empty_fields().join(", ")))?)
}


#[cfg(feature="document_export")]
fn show_json(selection: &StorageSelection) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(selection, |p| println!("{}", p.to_json()))?)
}

fn show_ical(selection: &StorageSelection) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(selection, |p| p.to_ical().print().unwrap())?)
}

fn show_detail(selection: &StorageSelection, detail: &str) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(selection, |p| {
        println!("{}", p.get(detail).unwrap_or_else(|| format!("No {:?} found", selection)))
    })?)
}

fn show_csv(selection: &StorageSelection) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(selection, |p| println!("{}",  p.to_csv(&BillType::Invoice).unwrap()) )?)
}

#[cfg(not(feature="document_export"))]
fn show_json(_: StorageDir, _: &[&str]) -> Result<()> {
    error!("feature temporarily disabled")?
}

pub fn show_path(matches: &ArgMatches) -> Result<()> {
    Ok(path(matches, |path| {
        println!("{}", path.display());
        Ok(())
    } )?)
}

/// Command SHOW --template
fn show_template(name: &str) -> Result<()> {
    let templater = Templater::from_file(&setup::<Project>()?.get_template_file(name)?)?;
    println!("{:#?}", templater.list_keywords());
    Ok(())
}

fn dump_yaml(selection: &StorageSelection) -> Result<()> {
    Ok(setup::<Project>()?.with_selection(selection, |p| println!("{}", p.dump_yaml()))?)
}

