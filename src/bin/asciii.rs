extern crate chrono;
extern crate term; // TODO consolidate term, ansi_term and terminal_size
extern crate open;

#[cfg(feature="shell")]
extern crate rustyline;

extern crate prettytable;
#[macro_use] extern crate log;
#[macro_use] extern crate clap;
#[macro_use] extern crate error_chain;

extern crate crowbook_intl_runtime;
#[macro_use] pub mod localize_macros;

extern crate asciii;

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

fn main(){
    asciii::util::setup_log();
    setup_locale();

    cli::with_cli(|app| cli::match_matches(&app.get_matches()) );
}
