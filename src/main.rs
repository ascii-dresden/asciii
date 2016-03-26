#![feature(plugin)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

#![cfg(not(doc))]
#![cfg(not(test))]
extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate tempdir;
extern crate term;
extern crate terminal_size;
//TODO make libgit2 optional
#[cfg(feature = "git")]
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

use clap::App;

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
    let cli_setup = init_matches(); //TODO Font forget this in production
    //let cli_setup = load_yaml!("cli/cli.yml");


    let matches = App::from_yaml(&cli_setup)
        .version(&crate_version!()[..])
        .get_matches();

    cli::subcommands::new(&matches);

    cli::subcommands::list(&matches);

    cli::subcommands::edit(&matches);

    cli::subcommands::show(&matches);

    cli::subcommands::archive(&matches);

    cli::subcommands::unarchive(&matches);

    cli::subcommands::config(&matches);

    cli::subcommands::path(&matches);

    cli::subcommands::git(&matches);


    if matches.is_present("term") {
        use terminal_size::{Width, Height, terminal_size };
        if let Some((Width(w), Height(h))) = terminal_size() {
            println!("Your terminal is {} cols wide and {} lines tall", w, h);
        } else {
            println!("Unable to get terminal size");
        }
    }

}

fn main(){
    setup_app();
}

