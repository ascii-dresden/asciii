#![warn(unused_import_braces, unused_qualifications, clippy::pedantic, clippy::doc_markdown)]
#![allow(clippy::non_ascii_literal, clippy::module_name_repetitions, clippy::use_self, clippy::must_use_candidate, clippy::if_not_else)]

#[macro_use] extern crate clap;

#[macro_use] pub mod localize_macros;

use asciii;

use std::env;

use crowbook_intl_runtime::set_lang;

pub mod cli;

fn setup_locale() {
    if let Ok(env_lang) = env::var("LANG") {
        if env_lang.starts_with("de") {
            set_lang("de");
        }
    }
}

fn main() {
    color_backtrace::install();
    asciii::util::setup_log();
    setup_locale();

    cli::with_cli(|app| cli::match_matches(&app.get_matches()) );
}
