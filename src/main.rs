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
use cli::subcommands;
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

fn setup_app(){
    trace!("setting up app");

    let matches = cli::setup();

    match matches.subcommand() {
     ("list",      Some(sub_m)) => subcommands::list(sub_m),
     ("csv",       Some(sub_m)) => subcommands::csv(sub_m),
     ("new",       Some(sub_m)) => subcommands::new(sub_m),
     ("edit",      Some(sub_m)) => subcommands::edit(sub_m),
     ("set",       Some(sub_m)) => subcommands::set(sub_m),
     ("show",      Some(sub_m)) => subcommands::show(sub_m),
     ("calendar",  Some(sub_m)) => subcommands::calendar(sub_m),
     ("archive",   Some(sub_m)) => subcommands::archive(sub_m),
     ("unarchive", Some(sub_m)) => subcommands::unarchive(sub_m),
     ("config",    Some(sub_m)) => subcommands::config(sub_m),
     ("whoami",    _          ) => subcommands::config_show("user/name"),

     ("path",      Some(sub_m)) => subcommands::show_path(sub_m),
     ("open",      Some(sub_m)) => subcommands::open_path(sub_m),

     ("make",      Some(sub_m)) => subcommands::make(sub_m),
     ("delete",    Some(sub_m)) => subcommands::delete(sub_m),
     ("spec",      Some(sub_m)) => subcommands::spec(sub_m),

     ("doc",       _          ) => subcommands::doc(),
     ("version",   _          ) => subcommands::version(),

     ("dues",      Some(sub_m)) => subcommands::dues(sub_m),

     ("remote",    _          ) => subcommands::git_remote(),
     ("pull",      Some(sub_m)) => subcommands::git_pull(sub_m),
     ("diff",      Some(sub_m)) => subcommands::git_diff(sub_m),
     ("cleanup",   Some(sub_m)) => subcommands::git_cleanup(sub_m),
     ("status",    _          ) => subcommands::git_status(),
     ("add",       Some(sub_m)) => subcommands::git_add(sub_m),
     //("unadd",     Some(sub_m)) => subcommands::git_unadd(sub_m),
     ("commit",    _          ) => subcommands::git_commit(),
     ("push",      _          ) => subcommands::git_push(),
     ("stash",     _          ) => subcommands::git_stash(),
     ("pop",       _          ) => subcommands::git_stash_pop(),
     ("log",       _          ) => subcommands::git_log(),
     _                          => ()
    }
}

fn main(){
    setup_log();
    setup_app();
}
