use std::env::current_dir;
use std::process::Command;
use std::process::Stdio;

fn main(){

    let args = vec![""];
    let paths = vec![current_dir().unwrap().join("Cargo.toml")];


    Command::new("vim")
        .args(&paths)
        .status()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });


}


