#![allow(dead_code)]
use std::io;
use std::env::{home_dir,current_dir};
use std::path::{Path,PathBuf};
use std::process::Command;

pub mod keyword_replacement;
pub use self::keyword_replacement::IsKeyword;

pub mod yaml;


pub fn freeze() {
    io::stdin().read_line(&mut String::new()).unwrap();
}

pub fn ls(path:&str){
    println!("tree {}", path);
    let output = Command::new("tree")
        .arg(&path)
        .output()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    println!("{}", String::from_utf8(output.stdout).unwrap());
}

/// TODO add something like this to the stdlib
/// TODO ~ must be first character
pub fn replace_home_tilde(p:&Path) -> PathBuf{
    let path = p.to_str().unwrap();
    PathBuf::from( path.replace("~",home_dir().unwrap().to_str().unwrap()))
}

#[export_macro]
macro_rules! try_some {
    ($expr:expr) => (match $expr {
        Some(val) => val,
        None => return None,
    });
}

//TODO use https://crates.io/crates/open (supports linux, windows, mac)
pub fn open_in_editor(editor:&str, paths:Vec<String>){
    let editor_config = editor
        .split_whitespace()
        .collect::<Vec<&str>>();

    let (editor_command,args) = editor_config
        .split_first().unwrap() ;

    println!("launching {:?} with {:?} and {:?}",
             editor_command,
             args.join(" "),
             paths);

    assert!(!paths.is_empty()); //TODO can I add a message to that?

    for path in &paths{
        assert!(Path::new(&path).exists());
    }

    Command::new(editor_command)
        .args(&args)
        .args(&paths)
        .status()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
}

pub fn get_storage_path() -> PathBuf
{
    let storage_path = PathBuf::from(super::CONFIG.get_str("path"))
        .join( super::CONFIG.get_str("dirs/storage"));

    // TODO make replace tilde a Trait function
    let mut storage_path = replace_home_tilde(&storage_path);

    if !storage_path.is_absolute(){
        storage_path = current_dir().unwrap().join(storage_path);
    }
    storage_path
}
