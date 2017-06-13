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
mod error;

#[cfg(feature="shell")]
pub mod shell;

pub use self::app::with_cli;
pub use self::app::match_matches;

/// prints a message and exist with code 1
pub fn fail<T:Display>(message:T) -> !{
    println!("{}", message);
    exit(1);
}
