extern crate yaml_rust;
extern crate chrono;
extern crate regex;
extern crate slug;
extern crate pad;
#[macro_use] extern crate lazy_static;

mod yaml;
pub mod config;
lazy_static!{
    pub static ref CONFIG: config::ConfigReader = config::ConfigReader::new().unwrap();
}
mod filter;
mod util;

pub mod project;
pub mod manager;
pub mod templater;
pub mod keyword_replacement;

pub use manager::Luigi;
pub use keyword_replacement::IsKeyword;

