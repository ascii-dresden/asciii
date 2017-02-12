//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::process::exit;
use std::fmt::Display;
use std::error::Error;

//use asciii::project::Project;
//use asciii::storage::*;
//use asciii::util;
//use asciii::CONFIG;

/// Contains concrete implementation of each subcommand
pub mod app;
pub mod subcommands;

#[cfg(feature="shell")]
pub mod shell;

pub use self::app::with_cli;
pub use self::app::match_matches;

/// prints a message and exist with code 1
pub fn fail<T:Display>(message:T) -> !{
    println!("{}", message);
    exit(1);
}

/// Execute a command returning a `StorageError`
pub fn execute<F, S, E:Error>(command:F) -> S where F: FnOnce() -> Result<S, E> {
    match command(){
        Ok(s) => s,
        Err(lerr) => {
            error!("{}", lerr);
            if let Some(cause) = lerr.cause() {
                println!("caused by {}", cause);
            }
            exit(1)
        }
    }
}
