#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]

#![cfg_attr(feature = "nightly", feature(alloc_system))]

#[feature(deprecated)]

#[cfg(feature = "nightly")]
extern crate alloc_system;

extern crate open;
extern crate yaml_rust;

#[macro_use]
extern crate clap;
use clap::App;

extern crate asciii;

use asciii::util;
use asciii::config;
use asciii::manual;
use asciii::project;
use asciii::storage;
use asciii::repo;
use asciii::templater;
use asciii::cli;


// TODO: add logging
// TODO: make better use of io::ErrorKind
// TODO: remove: to_owned() and unwrap()s, stupid :D

pub fn setup_app(){
    use asciii::cli::subcommands::*;
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
     ("whoami",    _          ) => config_show("storage_name"),

     ("path",      Some(sub_m)) => path(&sub_m, |path| println!("{:?}", path)),
     ("open",      Some(sub_m)) => path(&sub_m, |path| {open::that(path).unwrap();}),

     ("term",      _          ) => term(),
     ("doc",       _          ) => doc(),

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

