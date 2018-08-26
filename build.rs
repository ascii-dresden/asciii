extern crate chrono;
extern crate crowbook_intl;

use std::path::PathBuf;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Output};

use chrono::prelude::*;
use crowbook_intl::{Localizer, Extractor};

fn execute_git(command:&str, args:&[&str]) -> Output {
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

fn gen_commit_file() {
    let git_log    = String::from_utf8(execute_git("log", &["--oneline", r##"--format=%h"##]).stdout).unwrap();
    let count = git_log.lines().count().to_string();
    let last_commit= git_log.lines().nth(0).unwrap().to_string();
    let description = format!("commit {} ({})", count.trim(), last_commit.trim());
    println!("description string= {:?}",description);
    let mut f = File::create(".most_recent_commit").unwrap();
    f.write_all(description.as_bytes()).unwrap();
}

fn generate_localization() {
    // Generate a `lang/default.pot` containing strings used to call `lformat!`
    let mut extractor = Extractor::new();
    extractor.add_messages_from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).unwrap();
    extractor.write_pot_file(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/default.pot")).unwrap();

    // Generate the `localize_macros.rs` file
    let mut localizer = Localizer::new(&extractor);
    localizer.add_lang("de", include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/de.po"))).unwrap();
    // Use env::var instead of env! to avoid problems when cross-compiling
    let dest_path = PathBuf::from(env::var_os("OUT_DIR").unwrap())
       .join("localize_macros.rs");
    localizer.write_macro_file(dest_path).unwrap();
}

fn build_webapp() {
    Command::new("yarn")
        .current_dir("./webapp")
        .output()
        .unwrap_or_else(|e| { panic!("yarn install step failed {}", e) });

    Command::new("yarn")
        .current_dir("./webapp")
        .arg("build")
        .output()
        .unwrap_or_else(|e| { panic!("yarn build step failed {}", e) });
}

fn main() {
    // passing variables to rustc
    println!("cargo:rustc-env=PROFILE={}", env::var("PROFILE").unwrap_or("unknown profile".into()));
    println!("cargo:rustc-env=BUILD_DATE={}", Utc::now().format("%+"));

    if env::var("CARGO_FEATURE_WEBAPP") == Ok(String::from("1")) {
        build_webapp();
    }
    if env::var("CARGO_FEATURE_LOCALIZE") == Ok(String::from("1")) {
        generate_localization();
    }
    if env::var("CARGO_FEATURE_VERSION_STRING") == Ok(String::from("1")) {
        gen_commit_file();
    }
}
