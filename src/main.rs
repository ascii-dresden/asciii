#![cfg_attr(feature = "lints", allow(unstable_features))]
#![cfg_attr(feature = "lints", feature(plugin))]
#![cfg_attr(feature = "lints", plugin(clippy))]

extern crate chrono;
extern crate term; // TODO consolidate term, ansi_term and terminal_size
extern crate open;
extern crate icalendar;

#[cfg(feature="shell")]
extern crate rustyline;

#[cfg(feature="document_export")]
extern crate rustc_serialize;

extern crate env_logger;
extern crate prettytable;
#[macro_use] extern crate log;
#[macro_use] extern crate clap;
#[macro_use] extern crate maplit;

extern crate crowbook_intl_runtime;
#[macro_use] pub mod localize_macros;

extern crate asciii;

use std::env;

use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;
use crowbook_intl_runtime::set_lang;

pub mod cli;
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

fn setup_locale() {
    if let Ok(env_lang) = env::var("LANG") {
        if env_lang.starts_with("de") {
            set_lang("de");
        }
    }
}

fn main(){
    setup_log();
    setup_locale();

    cli::with_cli(|app| cli::match_matches(&app.get_matches()));
}
