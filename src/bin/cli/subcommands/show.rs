use clap::ArgMatches;

use asciii::print;
use asciii::storage::*;
//use asciii::storage::error::*;

use asciii::project::{spec, BillType, Project};
use asciii::project::spec::HasEvents;
use asciii::templater::Templater;

use super::{matches_to_search, matches_to_selection};

use super::path;
use cli::error::*;

use std::fs;

/// Command SHOW
pub fn show(m: &ArgMatches) -> Result<()> {
    let (search_terms, _) = matches_to_search(m);
    let selection = matches_to_selection(m);

    let bill_type = match (m.is_present("offer"), m.is_present("invoice")) {
        (true, true) => unreachable!("this should have been prevented by clap-rs"),
        (true, false) => BillType::Offer,
        // (false,true) => BillType::Invoice,
        _ => BillType::Invoice, //TODO be inteligent here ( use date )
    };

    if m.is_present("files") {
        show_files(selection)
    } else if let Some(detail) = m.value_of("detail") {
        show_detail(selection, detail)
    } else if m.is_present("empty fields") {
        show_empty_fields(selection)
    } else if m.is_present("errors") {
        show_errors(selection)
    } else if m.is_present("yaml") {
        show_yaml(selection)
    } else if m.is_present("json") {
        show_json(selection)
    } else if m.is_present("ical") {
        show_ical(selection)
    } else if m.is_present("csv") {
        show_csv(selection)
    } else if m.is_present("template") {
        show_template(search_terms[0])
    } else {
        for p in setup::<Project>()?.open_projects(selection)? {
            print::show_details(&p, bill_type)
        }
        Ok(())
    }
}

fn show_files(selection: StorageSelection) -> Result<()> {
    for project in setup::<Project>()?.open_projects(selection)? {
        println!("{}: ", project.dir().display());
        for entry in fs::read_dir(project.dir()).unwrap() {
            println!("  {}", entry.unwrap().path().display())
        }
    }
    Ok(())
}

fn show_errors(selection: StorageSelection) -> Result<()> {
    for p in setup::<Project>()?.open_projects(selection)? {
        println!("{}: ", p.short_desc());
        spec::print_specresult("offer", &p.is_ready_for_offer());
        spec::print_specresult("invoice", &p.is_ready_for_invoice());
        spec::print_specresult("archive", &p.is_ready_for_archive());
    }
    Ok(())
}

fn show_empty_fields(selection: StorageSelection) -> Result<()> {
    for p in setup::<Project>()?.open_projects(selection)? {
        println!("{}: {}", p.short_desc(), p.empty_fields().join(", "))
    }
    Ok(())
}


fn show_json(selection: StorageSelection) -> Result<()> {
    for p in setup::<Project>()?.open_projects(selection)? {
        println!("{}", p.to_json()?)
    }
    Ok(())
}

fn show_yaml(selection: StorageSelection) -> Result<()> {
    for p in setup::<Project>()?.open_projects(selection)? {
        println!("{}", p.dump_yaml())
    }
    Ok(())
}

fn show_ical(selection: StorageSelection) -> Result<()> {
    for p in setup::<Project>()?.open_projects(selection)? {
        p.to_ical().print()?
    }
    Ok(())
}

fn show_detail(selection: StorageSelection, detail: &str) -> Result<()> {
    for p in setup::<Project>()?.open_projects(&selection)? {
        println!("{}",
                 p.field(detail)
                  .unwrap_or_else(|| format!("No {:?} found", selection)))
    }
    Ok(())
}

fn show_csv(selection: StorageSelection) -> Result<()> {
    for p in setup::<Project>()?.open_projects(selection)? {
        println!("{}", p.to_csv(&BillType::Invoice)?)
    }
    Ok(())
}

pub fn show_path(matches: &ArgMatches) -> Result<()> {
    Ok(path(matches, |path| {
        println!("{}", path.display());
        Ok(())
    })?)
}

/// Command SHOW --template
fn show_template(name: &str) -> Result<()> {
    let templater = Templater::from_file(&setup::<Project>()?.get_template_file(name)?)?;
    println!("{:#?}", templater.list_keywords());
    Ok(())
}
