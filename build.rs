use std::process::{Command,Output};

//#[macro_use] extern crate clap;
//use clap::Shell;
//include!("src/cli/app.rs");
//fn gen_completions(){
//    let mut app = build_cli();
//    app.gen_completions("asciii",
//                        Shell::Bash,
//                        ".")
//
//}

fn execute_git(command:&str, args:&[&str]) -> Output{
    let workdir = ".";
    let gitdir  = "./.git";

    Command::new("git")
        .arg("--no-pager")
        .args(&["--work-tree", workdir])
        .args(&["--git-dir",   gitdir])
        .arg(command)
        .args(args)
        .output()
        .unwrap_or_else(|e| { panic!("git_failed {}", e) })
}

use std::fs::File;
use std::io::Write;

fn gen_commit_file(){
    let git_log    = String::from_utf8(execute_git("log", &["--oneline", r##"--format=%h"##]).stdout).unwrap();
    let count = git_log.lines().count().to_string();
    let last_commit= git_log.lines().nth(0).unwrap().to_string();
    let description = format!("build {} ({})", count.trim(), last_commit.trim());
    println!("description string= {:?}",description);
    let mut f = File::create(".most_recent_commit").unwrap();
    f.write_all(description.as_bytes()).unwrap();
}

extern crate crowbook_intl;
use std::path::Path;
use std::env;
use crowbook_intl::{Localizer, Extractor};
 
fn generate_localization() {
    // Generate a `lang/default.pot` containing strings used to call `lformat!`
    let mut extractor = Extractor::new();
    extractor.add_messages_from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap();
    extractor.write_pot_file(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/default.pot")).unwrap();

    // Generate the `localize_macros.rs` file
    let mut localizer = Localizer::new(&extractor);
    localizer.add_lang("de", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/de.po"))).unwrap();
    // Use env::var instead of env! to avoid problems when cross-compiling
    let dest_path = Path::new(&env::var("OUT_DIR").unwrap())
       .join("localize_macros.rs");
    localizer.write_macro_file(dest_path).unwrap();
}

fn main(){
    //gen_commit_file();
    //gen_completions();
    generate_localization();
}
