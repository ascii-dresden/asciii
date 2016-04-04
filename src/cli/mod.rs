//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::process::exit;
use project::Project;
use manager::{Luigi, LuigiDir, LuigiResult};
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

fn setup_luigi_with_git() -> Luigi {
    execute(||Luigi::new_with_git(util::get_storage_path(), "working", "archive", "templates"))
}

fn setup_luigi() -> Luigi {
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
            details:      None,
        }

    }
}

//TODO make this generic over LuigiProject
//TODO this belongs into ::manger::
pub struct ProjectList{
    pub projects: Vec<Project>
}

impl ProjectList{
    pub fn open_dir(dir:LuigiDir) -> LuigiResult<ProjectList>{
        setup_luigi().list_project_files(dir)
            .map(|project_paths|
                 ProjectList{
                     projects: project_paths.iter()
                         .filter_map(|path| match Project::open(path){
                             Ok(project) => Some(project),
                             Err(err) => {
                                 println!("Erroneous Project: {}\n {}", path.display(), err);
                                 None
                             }
                         }).collect::<Vec<Project>>()
                 }
                )
    }

    pub fn filter_by_all(&mut self, filters:&[&str]){
        for filter in filters{
            self.filter_by(filter);
        }
    }

    pub fn filter_by(&mut self, filter:&str){
        self.projects.retain(|p|{
            let (key,val) = filter.split_at(filter.find(':').unwrap_or(0));
            p.matches_filter(&key, &val[1..])
        });
    }
}

use std::ops::{Deref, DerefMut};
impl Deref for ProjectList{
    type Target=Vec<Project>;
    fn deref(&self) -> &Vec<Project>{
        &self.projects
    }
}

impl DerefMut for ProjectList{
    fn deref_mut(&mut self) -> &mut Vec<Project>{
        &mut self.projects
    }
}

impl IntoIterator for ProjectList{
    type Item = Project;
    type IntoIter= ::std::vec::IntoIter<Project>;
    fn into_iter(self) -> Self::IntoIter{
        self.projects.into_iter()
    }
}
