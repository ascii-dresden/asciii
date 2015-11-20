#![allow(dead_code)]
#![allow(unused_imports)]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate pad;
#[macro_use]
extern crate clap;

#[allow(unused_imports)]
use clap::{App, SubCommand, Arg};
use manager::LuigiDir;

mod filter;
mod util;
mod yaml;

mod project;
mod manager;
mod templater;
mod keyword_replacement;
mod config;
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
                                          .short("a")
                                          .long("archive")
                                          .takes_value(true))
                                    )
                        .get_matches();

    if let Some(matches) = matches.subcommand_matches("list") {
        if let Some(archive) = matches.value_of("archive"){
            let archive = archive.parse::<i32>().unwrap();
            cli::list_projects(LuigiDir::Archive(archive));
        }
        else {
            cli::list_projects(LuigiDir::Working);
        }
    }
}
