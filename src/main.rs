#![allow(dead_code)]
#![allow(unused_imports)]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate pad;
#[macro_use] extern crate clap;
#[macro_use] extern crate lazy_static;

#[allow(unused_imports)]
use clap::{App, SubCommand, Arg};
use manager::LuigiDir;

mod yaml;
mod config;
lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}

mod filter;
mod util;

mod project;
mod manager;
mod templater;
mod keyword_replacement;
mod cli;


fn main(){
    let matches = App::new("ascii-invoicer")
        .version(&crate_version!()[..])
        .author("Hendrik Sollich <hendrik@hoodie.de>")
        .about("ascii-invoicer in rust")
        .arg_required_else_help(true)

        .subcommand(SubCommand::with_name("list")
                    .arg( Arg::with_name("archive")
                          .help("list archived projects")
                          .short("a").long("archive")
                          .takes_value(true))
                   )

        .subcommand(SubCommand::with_name("show")
                    .arg( Arg::with_name("search_term")
                          .help("search term, possibly event name")
                          .required(true))

                    .arg( Arg::with_name("archive")
                          .help("display a specific project")
                          .short("a").long("archive")
                          .takes_value(true))
                   )

        .subcommand(SubCommand::with_name("config")
                    .about("deal with your config file")

                    .arg( Arg::with_name("edit")
                          .help("edit your config")
                          .short("e").long("edit")
                          )

                    .arg( Arg::with_name("show")
                          .help("show a specific config value")
                          .short("s").long("show")
                          .takes_value(true))

                    .arg( Arg::with_name("show all")
                          .help("show a specific config value")
                          .short("a").long("all")
                          )
                   )
        .subcommand(SubCommand::with_name("whoami")
                   )

        .get_matches();

    // command: "list"
    if let Some(matches) = matches.subcommand_matches("list") {
        if let Some(archive) = matches.value_of("archive"){
            let archive = archive.parse::<i32>().unwrap();
            cli::list_projects(LuigiDir::Archive(archive));
        } else {
            cli::list_projects(LuigiDir::Working);
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
            cli::show_config(&path);
        }
        else if matches.is_present("edit"){
            cli::edit_config();
        }
        else if matches.is_present("show all"){
            cli::show_config_all();
        }
    }

    // command: "config"
    else if matches.is_present("whoami") {
        cli::show_config("manager_name");
    }
}
