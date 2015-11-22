#![allow(dead_code)]
#![allow(unused_imports)]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate pad;
#[macro_use] extern crate lazy_static;
extern crate itertools;
#[macro_use] extern crate clap;

use clap::{App, SubCommand, Arg};
use manager::LuigiDir;

mod yaml;
mod config;

mod filter;
mod util;

mod project;
mod manager;
mod templater;
mod keyword_replacement;
mod cli;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}

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
                   )

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
        .subcommand(SubCommand::with_name("whoami"))

        .get_matches();

    // command: "list"
    if let Some(matches) = matches.subcommand_matches("list") {
        if matches.is_present("templates"){
            cli::list_templates();
        } else if matches.is_present("all"){
            cli::list_all_projects();
        } else {
            if let Some(archive) = matches.value_of("archive"){
                let archive = archive.parse::<i32>().unwrap();
                cli::list_projects(LuigiDir::Archive(archive));
            } else {
                cli::list_projects(LuigiDir::Working);
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

    // command: "whoami"
    else if  matches.is_present("whoami") {
        cli::config_show("manager_name");
    }
}

