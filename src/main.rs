#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]

#![cfg_attr(feature = "nightly", feature(alloc_system))]

#[cfg(feature = "nightly")]
extern crate alloc_system;
extern crate chrono;
extern crate term; // TODO consolidate term, ansi_term and terminal_size
extern crate open;
extern crate icalendar;

#[cfg(feature="shell")] extern crate rustyline;

#[cfg(feature="document_export")] extern crate rustc_serialize;

extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate clap;
#[macro_use] extern crate prettytable;

extern crate asciii;

use std::env;

use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;

pub mod cli;
use cli::match_matches;
pub mod manual;

fn setup_log(){
    let format = |record: &LogRecord| {
        format!("{level}:  {args}",
        level = record.level(),
        args  = record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Info);

    let log_var ="ASCIII_LOG";
    if env::var(log_var).is_ok() {
       builder.parse(&env::var(log_var).unwrap());
    }

    builder.init().unwrap();
}

fn main(){
    setup_log();

    trace!("setting up app");
    let matches = cli::build_cli().get_matches();
    match_matches(&matches);
}
