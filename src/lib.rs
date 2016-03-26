#![allow(dead_code)]
#![allow(unused_imports)]

#![cfg(doc)]
#![cfg(test)]
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


pub mod util;
pub mod config;

pub mod project;
pub mod manager;
pub mod repo;

pub mod templater;
pub mod cli;

pub use std::path::{Path,PathBuf};
pub use clap::App;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}
