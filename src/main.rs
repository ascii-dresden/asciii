#![allow(unused_variables)]
#![allow(dead_code)]

#![feature(plugin)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

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
use clap::App;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}

// TODO: add logging
// TODO: make better use of io::ErrorKind
// TODO: remove: to_owned() and unwrap()s, stupid :D

fn init_matches() -> yaml_rust::yaml::Yaml
{
    use std::fs::File;
    use std::io::Read;
    use util::yaml;

    // TODO replace this block with the line above
    println!("loading cli config at runtime!");
    let content = File::open("./src/cli/cli.yml")
        .and_then(|mut file| {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map(|_| content)
        }).unwrap();
    yaml::parse(&content).unwrap()
}

pub fn setup_app(){
    //let cli_setup = init_matches(); //TODO Font forget this in production
    let cli_setup = load_yaml!("cli/cli.yml");


    let matches = App::from_yaml(&cli_setup)
        .version(&crate_version!()[..])
        .get_matches();

    // command: "new"
    cli::subcommands::new(&matches);

    // command: "list"
    cli::subcommands::list(&matches);

    // command: "edit"
    cli::subcommands::edit(&matches);

    // command: "show"
    cli::subcommands::show(&matches);

    // command: "archive"
    cli::subcommands::archive(&matches);
    cli::subcommands::unarchive(&matches);
    cli::subcommands::config(&matches);

    // command: "path"
    if let Some(matches) = matches.subcommand_matches("path") {
        if matches.is_present("templates"){
            println!("{}", PathBuf::from(CONFIG.get_str("path"))
                     .join( CONFIG.get_str("dirs/storage"))
                     .join( CONFIG.get_str("dirs/templates"))
                     .display());
        }
        else if matches.is_present("output"){
            println!("{}", CONFIG.get_str("output_path"));
        }
        else if matches.is_present("bin"){
            println!("{}", std::env::current_exe().unwrap().display());
        }
        else { // default case
            let path = util::replace_home_tilde(Path::new(CONFIG.get_str("path")))
                .join( CONFIG.get_str("dirs/storage"));
            println!("{}", path.display());
        }
    }
    // command: "status"
    if matches.is_present("status") { cli::git_status(); }

    // command: "add"
    if matches.is_present("add") { cli::git_add(); }

    // command: "pull"
    if matches.is_present("pull") { cli::git_pull(); }

    if matches.is_present("remote") { cli::git_remote(); }


    if matches.is_present("term") {
        use terminal_size::{Width, Height, terminal_size };
        if let Some((Width(w), Height(h))) = terminal_size() {
            println!("Your terminal is {} cols wide and {} lines tall", w, h);
        } else {
            println!("Unable to get terminal size");
        }
    }

    if matches.is_present("remote") {
        cli::git_remote();
    }
}

fn main(){
    setup_app();
}

