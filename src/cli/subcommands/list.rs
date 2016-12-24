use clap::ArgMatches;
use chrono::*;

use asciii::CONFIG;
use asciii::actions::{setup_luigi, setup_luigi_with_git};
use asciii::storage::*;
use asciii::print;
use asciii::print::{ListConfig, ListMode};
use asciii::project::Project;
use asciii::project::spec::IsProject;
use asciii::project::ComputedField;

use ::cli::execute;

use std::path::PathBuf;

/// Command LIST
pub fn list(matches: &ArgMatches) {
    if matches.is_present("templates") {
        list_templates();
    } else if matches.is_present("years") {
        list_years();
    } else if matches.is_present("computed_fields") {
        list_computed_fields();
    } else {
        let list_mode = decide_mode(matches.is_present("simple"),
                                    matches.is_present("verbose"),
                                    matches.is_present("paths"),
                                    matches.is_present("nothing"),
                                    matches.is_present("csv"));

        let extra_details = matches.values_of("details").map(|v| v.collect::<Vec<&str>>());
        let config_details = CONFIG.get_strs("list/extra_details");

        let mut list_config = ListConfig {
            sort_by: matches.value_of("sort")
                            .unwrap_or_else(|| {
                                CONFIG.get_str("list/sort")
                                    .expect("Faulty config: field list/sort does not contain a string value")
                            }),
            mode: list_mode,
            details: extra_details.or(config_details),
            filter_by: matches.values_of("filter").map(|v| v.collect::<Vec<&str>>()),
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
        let dir =
            if matches.is_present("archive"){
                let archive_year = matches.value_of("archive")
                    .and_then(|y|y.parse::<i32>().ok())
                    .unwrap_or(UTC::today().year());
                StorageDir::Archive(archive_year)
            }

            else if matches.is_present("year"){
                let year = matches.value_of("year")
                    .and_then(|y|y.parse::<i32>().ok())
                    .unwrap_or(UTC::today().year());
                StorageDir::Year(year)
            }

            // or list all, but sort by date
            else if matches.is_present("all"){
                // sort by date on --all of not overriden
                if !matches.is_present("sort"){ list_config.sort_by = "date" }
                StorageDir::All }

            // or list normal
            else { StorageDir::Working };

        if matches.is_present("broken"){
            list_broken_projects(dir); // XXX Broken
        } else {
            list_projects(dir, &list_config);
        }
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
fn list_projects(dir: StorageDir, list_config: &ListConfig) {
    let luigi = if CONFIG.get_bool("list/gitstatus") {
        execute(setup_luigi_with_git)
    } else {
        execute(setup_luigi)
    };
    debug!("listing projects: {}", luigi.working_dir().display());

    let mut projects = execute(|| luigi.open_projects(dir));

    // filtering, can you read this
    if let Some(ref filters) = list_config.filter_by {
        projects.filter_by_all(filters);
    }

    // sorting
    match list_config.sort_by {
        "manager" => projects.sort_by(|pa,pb| pa.responsible().cmp( &pb.responsible())),
        "date"    => projects.sort_by(|pa,pb| pa.modified_date().cmp( &pb.modified_date())),
        "name"    => projects.sort_by(|pa,pb| pa.short_desc().cmp( &pb.short_desc())),
        "index"   => projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned()))), // TODO rename to ident
        _         => projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned()))),
    }

    // fit screen
    let wide_enough = true;

    if !wide_enough && list_config.mode != ListMode::Csv {
        // TODO room for improvement
        print::print_projects(print::simple_rows(&projects, list_config));
    } else {
        debug!("list_mode: {:?}", list_config.mode );
        match list_config.mode{
            ListMode::Csv     => print::print_csv(&projects),
            ListMode::Paths   => print::print_projects(print::path_rows(&projects, list_config)),
            ListMode::Simple  => print::print_projects(print::simple_rows(&projects, list_config)),
            ListMode::Verbose => print::print_projects(print::verbose_rows(&projects,list_config)),
            ListMode::Nothing => print::print_projects(print::dynamic_rows(&projects,list_config)),
        }
    }
}

/// Command LIST --broken
fn list_broken_projects(dir: StorageDir) {
    let luigi = execute(setup_luigi);
    let invalid_files = execute(|| luigi.list_project_files(dir));
    let tups = invalid_files.iter()
                            .filter_map(|dir| Project::open(dir).err().map(|e| (e, dir)))
                            .collect::<Vec<(StorageError, &PathBuf)>>();

    for (err, path) in tups {
        println!("{}: {:?}", path.display(), err);
    }
}

/// Command LIST --templates
fn list_templates() {
    let luigi = execute(setup_luigi);

    for name in execute(|| luigi.list_template_names()) {
        println!("{}", name);
    }
}

/// Command LIST --years
pub fn list_years() {
    let luigi = execute(setup_luigi);
    let years = execute(|| luigi.list_years());
    println!("{:?}", years);
}

/// Command LIST --virt
pub fn list_computed_fields() {
    println!("{:?}",
             ComputedField::iter_variant_names()
                 .filter(|v| *v != "Invalid")
                 .collect::<Vec<&str>>());
}

//#[deprecated(note="move to impl ListMode and then to asciii::actions")]
fn decide_mode(simple:bool, verbose:bool, paths:bool,nothing:bool, csv:bool) -> ListMode {
    if csv{     ListMode::Csv }
    else if nothing{ ListMode::Nothing }
    else if paths{   ListMode::Paths }
    else {
        match (simple, verbose, CONFIG.get_bool("list/verbose")){
            (false, true,  _   ) => {debug!("-v overwrites config"); ListMode::Verbose },
            (false,    _, true ) => {debug!("-v from config");ListMode::Verbose},
                          _      => {debug!("simple mode");ListMode::Simple},
        }
    }
}

