#![allow(dead_code)]
#![allow(unused_imports)]
#![cfg(not(test))]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate tempdir;
extern crate term;
extern crate git2;
extern crate currency;
#[macro_use] extern crate prettytable;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
#[macro_use] extern crate clap;

mod util;
mod config;

mod project;
mod manager;
mod repo;

mod templater;
mod cli;

use std::path::{Path,PathBuf};
use clap::{App, SubCommand, Arg, AppSettings};
use manager::LuigiDir;
use cli::SortOptions;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}

// TODO: add logging
// TODO: make better use of io::ErrorKind
// TODO: remove: to_owned() and unwrap()s, stupid :D

pub fn setup_app(){
    let matches = App::new("ascii-invoicer")
        .version(&crate_version!()[..])
        .author("Hendrik Sollich <hendrik@hoodie.de>")
        .about("The ascii invoicer III")
        .setting(AppSettings::ArgRequiredElseHelp)
        //.arg_required_else_help(true)

        .subcommand(SubCommand::with_name("list")

                    .arg( Arg::with_name("archive")
                          .help("list archived projects")
                          .short("a").long("archive")
                          .takes_value(true))

                    .arg( Arg::with_name("sort")
                          .help("sort by [date | index | name | manager]")
                          .short("s").long("sort")
                          //.possible_values(vec![ String::from("date"), String::from("index"), String::from("name"), String::from("manager") ])
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

        .subcommand(SubCommand::with_name("archive"))
        .subcommand(SubCommand::with_name("unarchive"))

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

        .subcommand(SubCommand::with_name("path"))
        .subcommand(SubCommand::with_name("whoami"))

        .get_matches();
    //let matches = App::from_yaml(load_yaml!("cli.yml"));

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
            cli::list_templates(); }
        else {

        let mut sort = matches.value_of("sort").unwrap_or("index");

        // list archive of year `archive`
        let dir = if let Some(archive_year) = matches.value_of("archive"){
            let archive = archive_year.parse::<i32>().unwrap();
            LuigiDir::Archive(archive)
        }

        // or list all, but sort by date
        else if matches.is_present("all"){
            // sort by date on --all of not overriden
            if !matches.is_present("sort"){ sort = "date" }
            LuigiDir::All }

        // or list normal
        else { LuigiDir::Working };

        cli::list_projects(dir, sort);
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

    // command: "path"
    else if  matches.is_present("path") {
        println!("{}", PathBuf::from(
                CONFIG.get_str("path"))
            .join( CONFIG.get_str("dirs/storage"))
            .display());
    }

    // command: "whoami"
    else if  matches.is_present("whoami") {
        cli::config_show("manager_name");
    }
}

fn main(){
    setup_app();
}

