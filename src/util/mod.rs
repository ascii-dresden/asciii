//! Utility functions that are needed all over the places.
#![allow(dead_code)]
use std::io;
use std::env::{home_dir,current_dir};
use std::ffi::OsStr;
use std::path::{Path,PathBuf};
use std::process;
use std::process::{Command, ExitStatus};

use open;

pub mod yaml;

#[export_macro]
macro_rules! try_some {
    ($expr:expr) => (match $expr {
        Some(val) => val,
        None => return None,
    });
}

/// Freezes the program until for inspection
pub fn freeze() {
    io::stdin().read_line(&mut String::new()).unwrap();
}

/// Asks for confirmation
pub fn really(msg:&str) -> bool {
    println!("{}", msg);
    let mut answer = String::new();
    if io::stdin().read_line(&mut answer).is_err(){ return false; }
    ["yes", "y",
    "j", "ja",
    "oui", "si", "da"]
        .contains(&answer.trim())
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
pub fn pass_to_command<T:AsRef<OsStr>>(editor:&Option<&str>, paths:&[T]){

    let paths = paths.iter()
                      .map(|o|PathBuf::from(&o))
                      .filter(|p|p.exists())
                      .collect::<Vec<PathBuf>>();


    if paths.is_empty() {
        warn!("non of the provided paths could be found")
    } else if paths.len() > 4{
        println!("you are a about to open {} files\n{:#?}", paths.len(), paths);
    } else {

        if let Some(ref editor) = *editor {
            let editor_config = editor
                .split_whitespace()
                .collect::<Vec<&str>>();

            let (editor_command,args) = editor_config.split_first().unwrap() ;
            info!("launching {:?} with {:?} and {:?}",
                  editor_command,
                  args.join(" "),
                  paths);

            Command::new(editor_command)
                .args(args)
                .args(&paths)
                .status()
                .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });

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
    let storage_path = PathBuf::from(::CONFIG.get_str("path")
            .expect("Faulty config: field path does not contain a string value"))
        .join( ::CONFIG.get_str("dirs/storage")
            .expect("Faulty config: field dirs/storage does not contain a string value"));

    // TODO make replace tilde a Trait function
    let storage_path = replace_home_tilde(&storage_path);

    if !storage_path.is_absolute(){
        current_dir().unwrap().join(storage_path)
    } else {
        storage_path
    }
}

/// takes a path that could be relative or contains a `~` and turn it into a path that exists
pub fn get_valid_path<T:AsRef<OsStr>>(p:T) -> Option<PathBuf>{
    let path = replace_home_tilde(Path::new(&p));
    let path = if !path.is_absolute(){
        current_dir().unwrap().join(path)
    } else { path };

    if path.exists() {
        Some(path)
    } else {None}
}

/// Exits with the exit status of a child process.
pub fn exit(status:ExitStatus) -> !{
    process::exit(status.code().unwrap_or(1));
}

use bill::Currency;

/// One place to decide how to display currency
pub fn currency_to_string(currency:&Currency) -> String {
    currency.postfix().to_string()
}


