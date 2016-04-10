//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::process::exit;
use project::Project;
use manager::{Luigi, LuigiProject, LuigiResult};
use util;
use ::CONFIG;

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

fn setup_luigi_with_git() -> Luigi<Project> {
    execute(||Luigi::new_with_git(util::get_storage_path(), "working", "archive", "templates"))
}

fn setup_luigi() -> Luigi<Project> {
    execute(|| Luigi::new(util::get_storage_path(), "working", "archive", "templates"))
}

/// Configuration for this list output.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ListConfig<'a>{
    verbose:      bool,
    simple:       bool,
    show_errors:  bool,
    git_status:   bool,
    sort_by:      &'a str,
    filter_by:    Option<Vec<&'a str>>,
    use_colors:   bool,
    paths:        bool,
    details:      Option<Vec<&'a str>>,
}

impl<'a> Default for ListConfig<'a>{
    fn default() -> ListConfig<'a>{
        ListConfig{
            verbose:      CONFIG.get_bool("list/verbose"),
            simple:       false,
            git_status:   CONFIG.get_bool("list/gitstatus"),
            show_errors:  false,
            sort_by:      CONFIG.get_str("list/sort"),
            filter_by:    None,
            use_colors:   CONFIG.get_bool("list/colors"),
            paths:        false,
            details:      None,
        }

    }
}

fn sort_by(projects:&mut Vec<Project>, option:&str){
    match option {
        "manager" => projects.sort_by(|pa,pb| pa.manager().cmp( &pb.manager())),
        "date"    => projects.sort_by(|pa,pb| pa.date().cmp( &pb.date())),
        "name"    => projects.sort_by(|pa,pb| pa.name().cmp( &pb.name())),
        "index"   => projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned()))) ,
        _         => projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned()))) ,
    }
}
