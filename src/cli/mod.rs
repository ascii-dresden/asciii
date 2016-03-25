//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::process::exit;

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

/// Command STATUS
pub fn git_status(){
    let luigi = setup_luigi_with_git();
    println!("{:#?}", luigi);
    println!("{:#?}", luigi.repository.unwrap().statuses);
}

/// Command REMOTE
/// exact replica of `git remote -v`
pub fn git_remote(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap().repo;

    for remote_name in repo.remotes().unwrap().iter(){ // Option<Option<&str>> oh, boy
        if let Some(name) = remote_name{
            if let Ok(remote) = repo.find_remote(name){
            println!("{}  {} (fetch)\n{}  {} (push)",
                    remote.name().unwrap_or("no name"),
                    remote.url().unwrap_or("no url"),
                    remote.name().unwrap_or("no name"),
                    remote.pushurl().or(remote.url()).unwrap_or(""),
                    );
            }else{println!("no remote")}
        }else{println!("no remote name")}
    }
}

/// Command ADD
pub fn git_add(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    //repo.add();
}

/// Command PULL
pub fn git_pull(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    repo.pull();
}
