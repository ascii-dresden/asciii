#![allow(dead_code)]
#![allow(unused_imports)]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate itertools;
extern crate tempdir;
extern crate term;
extern crate git2;
#[macro_use] extern crate prettytable;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
#[macro_use] extern crate clap;

mod util;
mod config;

mod project;
mod manager;

mod templater;
mod cli;

use clap::{App, SubCommand, Arg};
use manager::LuigiDir;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}

// TODO: add logging
// TODO: make better use of io::ErrorKind
// TODO: remove: asserts!, is_ok(), to_owned() and unwrap()s, stupid :D

fn main(){
    let matches = App::new("ascii-invoicer")
        .version(&crate_version!()[..])
        .author("Hendrik Sollich <hendrik@hoodie.de>")
        .about("The ascii invoicer III")
        .arg_required_else_help(true)

        .subcommand(SubCommand::with_name("list")
                    //Options:
                    //  -a, [--archive=N]                    # list archived projects
                    //      [--archives], [--no-archives]    # lists all years for which there are archives
                    //      [--all], [--no-all]              # lists all projects, ever
                    //      [--templates], [--no-templates]  # lists all templates
                    //  -p, [--paths], [--no-paths]          # list paths to .yml files
                    //  -b, [--blockers], [--no-blockers]    # list blockers
                    //  -e, [--errors], [--no-errors]        # list errors
                    //  -s, [--simple], [--no-simple]        # overrides the verbose setting
                    //  -c, [--colors], [--no-colors]        # overrides the colors setting # Default: true
                    //  -f, [--filter=key:value]             # filter by manager, caterer etc (experimental)
                    //  -n, [--no-colors], [--no-no-colors]  # overrides the colors setting
                    //  -d, [--details=one two three]        # adds details
                    //      [--edit=one two three]           # open all listed files for edit
                    //      [--csv], [--no-csv]              # output as csv
                    //      [--sort=SORT]                    # sort by [date | index | name | manager] # Possible values: date, index, name, manager
                    //      [--final], [--no-final]          # list shows final sum
                    //      [--caterers], [--no-caterers]    # list caterers
                    //      [--wages], [--no-wages]          # list wages

                    .arg( Arg::with_name("archive")
                          .help("list archived projects")
                          .short("a").long("archive")
                          .takes_value(true))

                    .arg( Arg::with_name("all")
                          .help("List all projects, ever")
                          .long("all"))

                    .arg( Arg::with_name("templates")
                          .help("list templates")
                          .short("t").long("templates"))

                    .arg( Arg::with_name("broken")
                          .help("list broken projects (without project file)")
                          .short("b").long("broken"))

                    )

        .subcommand(SubCommand::with_name("edit")
                    .about("Edit a specific project")

                    .arg( Arg::with_name("search_term")
                          .help("Search term, possibly event name")
                          .required(true))

                    .arg( Arg::with_name("archive")
                          .help("Pick an archived project")
                          .short("a").long("archive")
                          .takes_value(true))

                    .arg( Arg::with_name("template")
                          .help("Edit a template (currently .tyml)")
                          .short("t").long("template"))

                    .arg( Arg::with_name("editor")
                          .help("Override the configured editor")
                          .short("e").long("editor")
                          .takes_value(true))
                   )

        .subcommand(SubCommand::with_name("show")
                    .about("Display a specific project")

                    .arg( Arg::with_name("search_term")
                          .help("Search term, possibly event name")
                          .required(true))

                    .arg( Arg::with_name("archive")
                          .help("Pick an archived project")
                          .short("a").long("archive")
                          .takes_value(true))

                    .arg( Arg::with_name("template")
                          .help("Show show fields in templates that are filled")
                          .short("t").long("template")
                          .conflicts_with("archive")
                          )
                   )

        .subcommand(SubCommand::with_name("new")
                    .arg( Arg::with_name("name")
                          .help("Project name")
                          .required(true))

                    .arg( Arg::with_name("template")
                          .help("Use a specific template")
                          .short("t").long("template")
                          .takes_value(true))

                    .arg( Arg::with_name("editor")
                          .help("Override the configured editor")
                          .short("e").long("editor")
                          .takes_value(true))

                    .arg( Arg::with_name("don't edit")
                          .help("Do not edit the file after creation")
                          .short("d"))

                    )

        //.subcommand(SubCommand::with_name("archive"))
        //.subcommand(SubCommand::with_name("unarchive"))

        .subcommand(SubCommand::with_name("config")
                    .about("Show and edit your config")

                    .arg( Arg::with_name("edit")
                          .help("Edit your config")
                          .short("e").long("edit")
                          )

                    .arg( Arg::with_name("show")
                          .help("Show a specific config value")
                          .short("s").long("show")
                          .takes_value(true))

                    .arg( Arg::with_name("default")
                          .help("Show default config")
                          .short("d").long("default")
                          )
                   )

        .subcommand(SubCommand::with_name("status"))

        .subcommand(SubCommand::with_name("whoami"))

        .get_matches();

    // command: "new"
    if let Some(matches) = matches.subcommand_matches("new") {
        let name     = matches.value_of("name").unwrap();
        let editor   = CONFIG.get_path("editor").unwrap().as_str().unwrap();

        let template = matches.value_of("template").or(
            CONFIG.get_path("template").unwrap().as_str()
            ).unwrap();

        cli::new_project(&name, &template, &editor, !matches.is_present("don't edit"));
    }

    // command: "list"
    else if let Some(matches) = matches.subcommand_matches("list") {
        if matches.is_present("templates"){
            cli::list_templates();
        } else if matches.is_present("all"){
            cli::list_all_projects();
        } else {
            let dir = if let Some(archive) = matches.value_of("archive"){
                let archive = archive.parse::<i32>().unwrap();
                LuigiDir::Archive(archive)
            } else {
                LuigiDir::Working
            };
            if matches.is_present("broken"){
                cli::list_broken_projects(dir);
            } else {
                cli::list_projects(dir);
            }
        }
    }

    // command: "edit"
    else if let Some(matches) = matches.subcommand_matches("edit") {
        let search_term = matches.value_of("search_term").unwrap();

        let editor = matches.value_of("editor").unwrap_or( CONFIG.get_path("editor").unwrap().as_str().unwrap());

        if matches.is_present("template"){
            cli::edit_template(search_term,&editor);
        } else {
            if let Some(archive) = matches.value_of("archive"){
                let archive = archive.parse::<i32>().unwrap();
                cli::edit_project(LuigiDir::Archive(archive), &search_term, &editor);
            } else {
                cli::edit_project(LuigiDir::Working, &search_term, &editor);
            }
        }
    }

    // command: "show"
    else if let Some(matches) = matches.subcommand_matches("show") {
        let search_term = matches.value_of("search_term").unwrap();
        if let Some(archive) = matches.value_of("archive"){
            let archive = archive.parse::<i32>().unwrap();
            cli::show_project(LuigiDir::Archive(archive), &search_term);
        } else if  matches.is_present("template"){
            cli::show_template(search_term);
        } else {
            cli::show_project(LuigiDir::Working, &search_term);
        }
    }

    // command: "config"
    else if let Some(matches) = matches.subcommand_matches("config") {
        if let Some(path) = matches.value_of("show"){
            cli::config_show(&path);
        }
        else if matches.is_present("edit"){
            let editor = CONFIG.get_path("editor").unwrap().as_str().unwrap();
            cli::config_edit(&editor); }
        else if matches.is_present("default"){ cli::config_show_default(); }
    }

    // command: "status"
    else if  matches.is_present("status") {
        cli::status();
    }

    // command: "whoami"
    else if  matches.is_present("whoami") {
        cli::config_show("manager_name");
    }
}

