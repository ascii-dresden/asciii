#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]

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
#[macro_use] extern crate prettytable;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
#[macro_use] extern crate clap;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate enum_derive;

pub mod util;
pub mod config;

pub mod project;
pub mod manager;
pub mod repo;

pub mod templater;
pub mod cli;

use clap::App;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}
