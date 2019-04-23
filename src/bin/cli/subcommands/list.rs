
use chrono::prelude::*;
use clap::ArgMatches;
use log::debug;
use failure::Error;

use asciii::CONFIG;
use asciii::print::{self, ListConfig, ListMode};
use asciii::project::{Project, ComputedField};
use asciii::project::spec::IsProject;
use asciii::storage::*;


/// Command LIST
pub fn list(matches: &ArgMatches<'_>) -> Result<(), Error> {
    if matches.is_present("templates") {
        list_templates()?; Ok(())
    } else if matches.is_present("years") {
        list_years()?; Ok(())
    } else if matches.is_present("computed_fields") {
        list_computed_fields()?; Ok(())
    } else {
        let list_mode = decide_mode(matches.is_present("simple"),
                                    matches.is_present("verbose"),
                                    matches.is_present("paths"),
                                    matches.is_present("nothing"),
                                    matches.is_present("csv"));

        let extra_details = matches.values_of("details")
                                   .map(Iterator::collect);
        let config_details = CONFIG.get_strs("list/extra_details");

        let mut list_config = ListConfig {
            sort_by: matches.value_of("sort")
                            .unwrap_or_else(|| CONFIG.get_str("list/sort")),
            mode: list_mode,
            details: extra_details.or(config_details),
            filter_by: matches.values_of("filter")
                              .map(Iterator::collect),
            show_errors: matches.is_present("errors"),

            ..Default::default()
        };

        if matches.is_present("colors") {
            list_config.use_colors = true;
        }
        if matches.is_present("no-colors") {
            list_config.use_colors = false;
        }

        // list archive of year `archive`
        let dir = if matches.is_present("archive") {
            let archive_year = matches.value_of("archive")
                                      .and_then(|y| y.parse::<i32>().ok())
                                      .unwrap_or_else(|| Utc::today().year());
            StorageDir::Archive(archive_year)
        } else if matches.is_present("year") {
            let year = matches.value_of("year")
                              .and_then(|y| y.parse::<i32>().ok())
                              .unwrap_or_else(|| Utc::today().year());
            StorageDir::Year(year)
        }
        // or list all, but sort by date
        else if matches.is_present("all") {
            // sort by date on --all of not overriden
            if !matches.is_present("sort") {
                list_config.sort_by = "date"
            }
            StorageDir::All
        }
        // or list normal
        else {
            StorageDir::Working
        };

        if matches.is_present("broken") {
               list_broken_projects(dir)?; // XXX Broken
           } else {
               list_projects(dir, &list_config)?;
           }
        Ok(())
    }
}

/// Command LIST [--archive, --all]
///
/// This interprets the `ListConfig` struct and passes it on to either
///
/// * `print::rows()`
/// * `print::simple_rows()`
/// * `print::verbose_rows()`
///
/// which it prints with `print::print_projects()`
fn list_projects(dir: StorageDir, list_config: &ListConfig<'_>) -> Result<(), Error> {
    let storage = if CONFIG.get_bool("list/gitstatus") {
        setup_with_git::<Project>()?
    } else {
        setup::<Project>()?
    };
    debug!("listing projects: {}", storage.working_dir().display());

    let mut projects = storage.open_projects(dir)?;

    // filtering, can you read this
    if let Some(ref filters) = list_config.filter_by {
        projects.filter_by_all(filters);
    }

    // sorting
    match list_config.sort_by {
        "manager" => projects.sort_by(|pa, pb| pa.responsible().cmp(&pb.responsible())),
        "date" => projects.sort_by(|pa, pb| pa.modified_date().cmp(&pb.modified_date())),
        "name" => projects.sort_by(|pa, pb| pa.short_desc().cmp(&pb.short_desc())),
        "index" => {
            projects.sort_by(|pa, pb| {
                                 pa.index()
                                   .unwrap_or_else(|| "zzzz".to_owned())
                                   .cmp(&pb.index().unwrap_or_else(|| "zzzz".to_owned()))
                             })
        } // TODO rename to ident
        _ => {
            projects.sort_by(|pa, pb| {
                                 pa.index()
                                   .unwrap_or_else(|| "zzzz".to_owned())
                                   .cmp(&pb.index().unwrap_or_else(|| "zzzz".to_owned()))
                             })
        }
    }

    // fit screen
    let wide_enough = true;

    if !wide_enough && list_config.mode != ListMode::Csv {
        // TODO room for improvement
        print::print_projects(print::simple_rows(&projects, list_config));
    } else {
        debug!("list_mode: {:?}", list_config.mode);
        match list_config.mode {
            ListMode::Csv => print::print_csv(&projects),
            ListMode::Paths => print::print_projects(print::path_rows(&projects, list_config)),
            ListMode::Simple => print::print_projects(print::simple_rows(&projects, list_config)),
            ListMode::Verbose => print::print_projects(print::verbose_rows(&projects, list_config)),
            ListMode::Nothing => print::print_projects(print::dynamic_rows(&projects, list_config)),
        }
    }
    Ok(())
}

/// Command LIST --broken
fn list_broken_projects(dir: StorageDir) -> Result<(), Error> {
    let storage = setup::<Project>()?;
    let invalid_files = storage.list_project_folders(dir)?;
    let errors = invalid_files.iter()
                            .filter_map(|dir| Project::open_folder(dir).err())
                            .collect::<Vec<failure::Error>>();

    for err in errors {
        println!("{}", err.as_fail());
    }
    Ok(())
}

/// Command LIST --templates
fn list_templates() -> Result<(), Error> {
    let storage = setup::<Project>()?;

    for name in storage.list_template_names()? {
        println!("{}", name);
    }
    Ok(())
}

/// Command LIST --years
pub fn list_years() -> Result<(), Error> {
    let storage = setup::<Project>()?;
    let years = storage.list_years()?;
    println!("{:?}", years);
    Ok(())
}

/// Command LIST --virt
pub fn list_computed_fields() -> Result<(), Error> {
    println!("{:?}",
             ComputedField::iter_variant_names()
                 .filter(|v| *v != "Invalid")
                 .collect::<Vec<&str>>());
    Ok(())
}

//#[deprecated(note="move to impl ListMode and then to asciii::actions")]
fn decide_mode(simple: bool, verbose: bool, paths: bool, nothing: bool, csv: bool) -> ListMode {
    if csv {
        ListMode::Csv
    } else if nothing {
        ListMode::Nothing
    } else if paths {
        ListMode::Paths
    } else {
        match (simple, verbose, CONFIG.get_bool("list/verbose")) {
            (false, true, _) => {
                debug!("-v overwrites config");
                ListMode::Verbose
            }
            (false, _, true) => {
                debug!("-v from config");
                ListMode::Verbose
            }
            _ => {
                debug!("simple mode");
                ListMode::Simple
            }
        }
    }
}
