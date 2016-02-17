#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(path_relative_from)]
#![feature(deprecated)]
#![cfg(not(test))]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate tempdir;
extern crate term;
extern crate terminal_size;
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

fn init_matches() -> yaml_rust::yaml::Yaml
{
    // TODO replace this block with the line above
    println!("loading cli config at runtime!");
    use std::fs::File;
    use std::io::Read;
    use yaml_rust::{Yaml};
    use util::yaml;
    let content = File::open("./src/cli/cli.yml")
        .and_then(|mut file| {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map(|_| content)
        }).unwrap();
    yaml::parse(&content).unwrap()
}

pub fn setup_app(){
    let cli_setup = init_matches(); //TODO Font forget this in production
    //let cli_setup = load_yaml!("cli/cli.yml");


    let matches = App::from_yaml(&cli_setup)
        .version(&crate_version!()[..])
        .get_matches();

    // command: "new"
    if let Some(matches) = matches.subcommand_matches("new") {
        let name     = matches.value_of("name").unwrap();
        let editor   = CONFIG.get_path("editor").unwrap().as_str().unwrap();

        let template = matches.value_of("template")
            .or( CONFIG.get_path("template").unwrap().as_str())
            .unwrap();

        cli::new_project(&name, &template, &editor, !matches.is_present("don't edit"));
    }

    // command: "list"
    else if let Some(matches) = matches.subcommand_matches("list") {
        if matches.is_present("templates"){
            cli::list_templates();
        } else {

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

            if matches.is_present("broken"){
                cli::list_broken_projects(dir);
            } else {
                cli::list_projects(dir, sort, matches.is_present("simple"));
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
        if let Some(year) = matches.value_of("archive"){
            let year = year.parse::<i32>().unwrap();
            cli::show_project(LuigiDir::Archive(year), &search_term);
        } else if  matches.is_present("template"){
            cli::show_template(search_term);
        } else {
            cli::show_project(LuigiDir::Working, &search_term);
        }
    }

    // command: "archive"
    else if let Some(matches) = matches.subcommand_matches("archive") {
        let name = matches.value_of("NAME").unwrap();
        let year = matches.value_of("year")
            .and_then(|s|s.parse::<i32>().ok());
        cli::archive_project(&name, year);
    }

    // command: "unarchive"
    else if let Some(matches) = matches.subcommand_matches("unarchive") {
        let year = matches.value_of("YEAR").unwrap();
        let name = matches.value_of("NAME").unwrap();
        let year = year.parse::<i32>().unwrap_or_else(|e|panic!("can't parse year {:?}, {:?}", year, e));
        cli::unarchive_project(year, &name);
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
    else if matches.is_present("path") {
        println!("{}", PathBuf::from(
                CONFIG.get_str("path"))
            .join( CONFIG.get_str("dirs/storage"))
            .display());
    }
    // command: "status"
    else if matches.is_present("status") { cli::git_status(); }

    // command: "pull"
    else if matches.is_present("pull") { cli::git_pull(); }

    // command: "whoami"
    else if matches.is_present("whoami") {
        cli::config_show("manager_name");
    }

    else if matches.is_present("term") {
        use terminal_size::{Width, Height, terminal_size };
        if let Some((Width(w), Height(h))) = terminal_size() {
            println!("Your terminal is {} cols wide and {} lines tall", w, h);
        } else {
            println!("Unable to get terminal size");
        }
    }

    else if matches.is_present("remote") {
        cli::git_remote();
    }
}

fn main(){
    setup_app();
}

