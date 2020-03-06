use std::env;
use std::path::PathBuf;
use std::process::Command;

use chrono::prelude::*;
use crowbook_intl::{Extractor, Localizer};

fn gen_commit_file() {
    let git_log_output = Command::new("git")
        .arg("--no-pager")
        .args(&["--work-tree", "."])
        .args(&["--git-dir", "./.git"])
        .arg("log")
        .args(&["--oneline", r##"--format=%h"##])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok());

    if let Some(git_log) = git_log_output {
        let count = git_log.lines().count().to_string();
        if let Some(last_commit) = git_log.lines().next() {
            println!(
                "cargo:rustc-env=GIT_DESCRIPTION=commit {} ({})",
                count.trim(),
                last_commit.to_string().trim()
            );
        } else {
            println!(r#"cargo:rustc-env=GIT_DESCRIPTION="no git description""#);
        }
    } else {
        println!(r#"cargo:rustc-env=GIT_DESCRIPTION="no git description""#);
    }
}

fn generate_localization() {
    // Generate a `lang/default.pot` containing strings used to call `lformat!`
    let mut extractor = Extractor::new();
    extractor
        .add_messages_from_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
        .unwrap();
    extractor
        .write_pot_file(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/default.pot"))
        .unwrap();

    // Generate the `localize_macros.rs` file
    let mut localizer = Localizer::new(&extractor);
    localizer
        .add_lang(
            "de",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lang/de.po")),
        )
        .unwrap();
    // Use env::var instead of env! to avoid problems when cross-compiling
    let dest_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("localize_macros.rs");
    localizer.write_macro_file(dest_path).unwrap();
}

fn build_webapp() {
    Command::new("yarn")
        .current_dir("./webapp")
        .output()
        .unwrap_or_else(|e| panic!("yarn install step failed {}", e));

    Command::new("yarn")
        .current_dir("./webapp")
        .arg("build")
        .output()
        .unwrap_or_else(|e| panic!("yarn build step failed {}", e));
}

fn main() {
    // passing variables to rustc
    println!(
        "cargo:rustc-env=PROFILE={}",
        env::var("PROFILE").unwrap_or_else(|_| "unknown profile".into())
    );
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
