//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::process::exit;
use std::path::PathBuf;

use manager::{Luigi, LuigiProject, LuigiResult};
use util;

/// Contains concrete implementation of each subcommand
pub mod subcommands ;
pub mod print;


/// Execute a command returning a LuigiError
/// TODO make this a `try!` like macro
fn execute<F, S>(command:F) -> S where F: FnOnce() -> LuigiResult<S> {
    match command(){
        Ok(s) => s,
        Err(lerr) => { println!("ERROR: {:?}", lerr); exit(1) }
    }
}

fn setup_luigi_with_git() -> Luigi {
    execute(||Luigi::new_with_git(util::get_storage_path(), "working", "archive", "templates"))
}

fn setup_luigi() -> Luigi {
    execute(|| Luigi::new(util::get_storage_path(), "working", "archive", "templates"))
}

