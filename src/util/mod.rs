#![allow(dead_code)]
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::env::home_dir;

pub mod keyword_replacement;
pub use self::keyword_replacement::IsKeyword;

pub mod yaml;


pub fn freeze() {
    let mut _devnull = String::new();
    let _ = io::stdin().read_line(&mut _devnull);
}

pub fn ls(path:&str){
    use std::process::Command;
    println!("tree {}", path);
    let output = Command::new("tree")
        .arg(&path)
        .output()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    println!("{}", String::from_utf8(output.stdout).unwrap());
}

/// TODO add something like this to the stdlib
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

use std::process::Command;
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
        .arg(args.join(" "))
        .args(&paths)
        .spawn()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
}
