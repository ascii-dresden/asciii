//! Thank you for choosing `asciii`, the new and improved `ascii`.
//!
//! This contains user and developer documentation together.
//! For pure user documentation please refer to the [user manual](manual/index.html).
//!

//#![warn(missing_docs,
//        missing_copy_implementations,
//        missing_debug_implementations
//        unstable_features,
//        unused_import_braces,
//        )]

#![deny(
    trivial_casts,
    trivial_numeric_casts,
    )]
#![warn(
    unstable_features,
    unused_import_braces,
    unused_qualifications
    )]

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
extern crate git2;
extern crate currency;
extern crate open;
#[macro_use] extern crate log;
#[macro_use] extern crate prettytable;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate enum_derive;
#[macro_use] extern crate clap;

#[macro_use]
pub mod util;
pub mod config;
pub mod manual;

pub mod project;
pub mod storage;
pub mod repo;

pub mod templater;
pub mod cli;

#[cfg(feature="document_export")]
pub mod fill_docs;

lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}
