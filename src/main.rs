#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]

#![feature(alloc_system)]
extern crate alloc_system;

extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate tempdir;
extern crate term; // TODO consolidate term, ansi_term and terminal_size
extern crate terminal_size;
//TODO make libgit2 optional
extern crate git2;
extern crate currency;
extern crate open;
#[macro_use] extern crate prettytable;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
#[macro_use] extern crate clap;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate enum_derive;

use clap::App;


#[macro_use]
mod util;
mod config;
mod doc;

mod project;
mod manager;
mod repo;

mod templater;
mod cli;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}

// TODO: add logging
// TODO: make better use of io::ErrorKind
// TODO: remove: to_owned() and unwrap()s, stupid :D

#[allow(dead_code)]
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
    use cli::subcommands::*;
    //let cli_setup = init_matches(); //TODO Font forget this in production
    let cli_setup = load_yaml!("cli/cli.yml");


    let matches = App::from_yaml(&cli_setup)
        .version(&crate_version!()[..])
        .get_matches();

    match matches.subcommand() {
     ("list",      Some(sub_m)) => list(&sub_m),
     ("new",       Some(sub_m)) => new(&sub_m),
     ("edit",      Some(sub_m)) => edit(&sub_m),
     ("show",      Some(sub_m)) => show(&sub_m),
     ("archive",   Some(sub_m)) => archive(&sub_m),
     ("unarchive", Some(sub_m)) => unarchive(&sub_m),
     ("config",    Some(sub_m)) => config(&sub_m),
     ("whoami",    _          ) => config_show("manager_name"),

     ("path",      Some(sub_m)) => path(&sub_m),

     ("term",      _          ) => term(),

     ("remote",    _          ) => git_remote(),
     ("pull",      _          ) => git_pull(),
     ("status",    _          ) => git_status(),
     ("add",       Some(sub_m)) => git_add(&sub_m),
     ("commit",    _          ) => git_commit(),
     ("push",      _          ) => git_push(),
     _                       => ()
    }


}

fn main(){
    setup_app();
}

