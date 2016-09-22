//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::process::exit;
use std::fmt::Display;

use asciii::project::Project;
use asciii::storage::*;
use asciii::util;
use asciii::CONFIG;

/// Contains concrete implementation of each subcommand
pub mod app;
pub mod subcommands;

pub use self::app::setup;

/// prints a message and exist with code 1
pub fn fail<T:Display>(message:T) -> !{
    println!("{}", message);
    exit(1);
}

/// Execute a command returning a `StorageError`
/// TODO make this a `try!` like macro
fn execute<F, S>(command:F) -> S where F: FnOnce() -> StorageResult<S> {
    match command(){
        Ok(s) => s,
        Err(lerr) => { error!("{}", lerr); exit(1) }
    }
}

/// Sets up an instance of `Storage`.
fn setup_luigi() -> Storage<Project> {
    trace!("setup_luigi()");
    let working = CONFIG.get_str("dirs/working")
                .expect("Faulty config: dirs/working does not contain a value");
    let archive = CONFIG.get_str("dirs/archive")
                .expect("Faulty config: dirs/archive does not contain a value");
    let templates = CONFIG.get_str("dirs/templates")
                .expect("Faulty config: dirs/templates does not contain a value");
    execute(|| Storage::new(util::get_storage_path(), working, archive, templates))
}

/// Sets up an instance of `Storage`, with git turned on.
fn setup_luigi_with_git() -> Storage<Project> {
    trace!("setup_luigi_with_git()");
    let working = CONFIG.get_str("dirs/working")
                .expect("Faulty config: dirs/working does not contain a value");
    let archive = CONFIG.get_str("dirs/archive")
                .expect("Faulty config: dirs/archive does not contain a value");
    let templates = CONFIG.get_str("dirs/templates")
                .expect("Faulty config: dirs/templates does not contain a value");
    execute(||Storage::new_with_git(util::get_storage_path(), working, archive, templates))
}



pub mod validators{
    use asciii::util::yaml::parse_dmy_date;

    pub fn is_dmy(val: String) -> Result<(),String>{
        match parse_dmy_date(&val){
            Some(_) => Ok(()),
            None => Err(String::from("Date Format must be DD.MM.YYYY")),
        }
    }
}
