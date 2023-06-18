#![allow(deprecated)]
#![allow(clippy::uninlined_format_args)]

#[macro_use]
pub mod localize_macros;

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

    cli::with_cli(|app| cli::match_matches(&app.get_matches()));
}
