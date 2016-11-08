//! Thank you for choosing `asciii`, the new and improved `ascii`.
//!
//! **For pure user documentation please refer to the [user manual](manual/index.html).**
//! This contains user and developer documentation together.
//!
//! Please check out [cli](cli/index.html) too.

//#![warn(missing_docs,
//        missing_copy_implementations,
//        missing_debug_implementations
//        unstable_features,
//        unused_import_braces,
//        )]

#![deny( trivial_casts, trivial_numeric_casts,)]

#![warn( unused_import_braces, unused_qualifications)]

#![allow(unknown_lints, needless_borrow)]
//#![allow(deprecated)]

#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]
//#![cfg_attr(feature = "lints", plugin(herbie_lint))]

//#![feature(alloc_system)]
//extern crate alloc_system;

extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate tempdir;
extern crate term;
extern crate bill;
extern crate ordered_float;
extern crate open;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;
#[macro_use] extern crate prettytable;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
#[macro_use] extern crate custom_derive;
#[macro_use] extern crate enum_derive;

#[cfg(feature="git_statuses")] extern crate git2;

#[macro_use]
pub mod util;
pub mod config;
pub mod manual;

pub mod project;
pub mod storage;
pub mod print;
pub mod actions;

pub mod templater;

#[cfg(feature="document_export")] extern crate rustc_serialize;
#[cfg(feature="document_export")] extern crate handlebars;
#[cfg(feature="document_export")] pub mod fill_docs;

pub use yaml_rust::Yaml;

// TODO keep this up to date after move
pub static DOCUMENTATION_URL: &'static str  = "http://ascii-dresden.github.io/asciii-rs/";

lazy_static!{
    /// Static `ConfigReader` to be able to access the configuration from everywhere.
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();

    /// Human readable, no semantic versioning.
    pub static ref VERSION: String = format!("{} - {}", env!("CARGO_PKG_VERSION"), include_str!("../.most_recent_commit"));

    /// Hint for app to point at `asciii::DOCUMENTATION_URL`
    pub static ref DOCHINT: String = format!("Documentation at: {}", DOCUMENTATION_URL);
}

#[derive(Debug)]
pub enum BillType{
    Offer,
    Invoice
}

impl ToString for BillType{
    fn to_string(&self) -> String {
        match *self{
            BillType::Offer => "Offer",
            BillType::Invoice => "Invoice"
        }.to_owned()
    }
}
