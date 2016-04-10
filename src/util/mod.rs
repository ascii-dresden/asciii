//! Utility functions that are needed all over the places.
#![allow(dead_code)]
use std::io;
use std::env::{home_dir,current_dir};
use std::path::{Path,PathBuf};
use std::process;
use std::process::{Command, ExitStatus};

use open;

pub mod yaml;


/// Freezes the program until for inspection
pub fn freeze() {
    io::stdin().read_line(&mut String::new()).unwrap();
}

/// Shells out to print directory structure
pub fn ls(path:&str){
    println!("tree {}", path);
    let output = Command::new("tree")
        .arg(&path)
        .output()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    println!("{}", String::from_utf8(output.stdout).unwrap());
}

/// Replaces `~` with `$HOME`, rust stdlib doesn't do this yet.
///
/// This is by far the most important function of all utility functions.
///
/// **TODO** add something like this to the stdlib
/// **TODO** ~ must be first character
pub fn replace_home_tilde(p:&Path) -> PathBuf{
    let path = p.to_str().unwrap();
    PathBuf::from( path.replace("~",home_dir().unwrap().to_str().unwrap()))
}

/// Opens the passed paths in the editor set int config.
///
/// This is by far the most important function of all utility functions.
//TODO use https://crates.io/crates/open (supports linux, windows, mac)
pub fn open_in_editor(editor:&Option<&str>, paths:&[PathBuf]){
    for path in paths{
        assert!(Path::new(&path).exists());
    }

    if let &Some(ref editor) = editor{
        let editor_config = editor
            .split_whitespace()
            .collect::<Vec<&str>>();

        let (editor_command,args) = editor_config.split_first().unwrap() ;

        println!("launching {:?} with {:?} and {:?}",
                 editor_command,
                 args.join(" "),
                 paths);

        assert!(!paths.is_empty()); //TODO can I add a message to that?

        Command::new(editor_command)
            .args(&args)
            .args(&paths)
            .status()
            .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    } else {
        if paths.len() > 4{
            println!("you are a about to open {} files\n{:#?}", paths.len(), paths);
        } else {
            for path in paths{
                open::that(path).unwrap();
            }
        }
    }
}

/// Interprets storage path from config.
///
/// Even if it starts with `~` or is a relatetive path.
/// This is by far the most important function of all utility functions.
pub fn get_storage_path() -> PathBuf
{
    let storage_path = PathBuf::from(::CONFIG.get_str("path"))
        .join( ::CONFIG.get_str("dirs/storage"));

    // TODO make replace tilde a Trait function
    let mut storage_path = replace_home_tilde(&storage_path);

    if !storage_path.is_absolute(){
        storage_path = current_dir().unwrap().join(storage_path);
    }
    storage_path
}

/// Exits with the exit status of a child process.
pub fn exit(status:ExitStatus) -> !{
    process::exit(status.code().unwrap_or(1));
}
