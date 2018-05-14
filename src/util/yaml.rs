//! Yaml Utility functions.
//!
//! Here are some missing batteries form the `yaml-rust` crate.
//! The cool thing about this is the simple path like access to nested elements.
//! if the yaml looks like this:
//!
//! ```yaml
//! programmer:
//!   name: Hendrik
//!   looks: good
//!   languages:
//!     * rust
//!     * ruby
//!     * python
//! ```
//!
//! you can access "ruby" like this: `get_string("programmer/languages/1")`.
//! Leading `/` will not be regarded.

#![allow(dead_code)]

use std::fmt;
use std::io::{self, Read};
use std::fs::File;
use std::path::Path;
use std::error::Error;

pub use yaml_rust::Yaml;
use yaml_rust::YamlLoader;
use yaml_rust::yaml::Hash as YamlHash;
use yaml_rust::scanner::ScanError;
use chrono::prelude::*;

/// Wrapper around `io::Error` and `yaml_rust::scanner::ScanError`.
#[derive(Debug)]
pub enum YamlError{
    /// wrapped `io` Error
    Io(io::Error),
    /// wrapped `scan` Error
    Scan(ScanError)
}

impl Error for YamlError{
    fn description(&self) -> &str{
        match *self{
            YamlError::Io(ref err) => err.description(),
            YamlError::Scan(ref err) => err.description()
        }
    }

    fn cause(&self) -> Option<&Error>{
        match *self{
            YamlError::Io(ref err) => err.cause(),
            YamlError::Scan(ref err) => err.cause()
        }
    }
}

impl From<io::Error> for YamlError { fn from(ioerror: io::Error)   -> YamlError{ YamlError::Io(ioerror) } }
impl From<ScanError> for YamlError { fn from(scanerror: ScanError) -> YamlError{ YamlError::Scan(scanerror) } }
impl fmt::Display for YamlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self{
            YamlError::Scan(ref err) => write!(f, "{}", err),
            YamlError::Io(ref err) => write!(f, "{}", err)
        }
    }
}

/// Wrapper that opens and parses a `.yml` file.
pub fn open(path: &Path) -> Result<Yaml, YamlError> {
    let file_content = File::open(&path)
                             .and_then(|mut file| {
                                 let mut content = String::new();
                                 file.read_to_string(&mut content).map(|_| content)
                             })?;
    parse( &file_content )
}

/// Ruby like API to yaml-rust.
pub fn parse(file_content: &str) -> Result<Yaml, YamlError> {
    Ok(
        YamlLoader::load_from_str(&file_content)?
        .get(0)
        .map(|i|i.to_owned())
        .unwrap_or_else(||Yaml::from_str("[]"))
      )
}

/// Interprets `"25.12.2016"` as date.
pub fn parse_dmy_date(date_str:&str) -> Option<Date<Utc>>{
    let date = date_str.split('.')
        .map(|f|f.parse().unwrap_or(0))
        .collect::<Vec<i32>>();
    if date.len() >=2 && date[0] > 0 && date[2] > 1900 {
        // XXX this neglects the old "01-05.12.2015" format
        Utc.ymd_opt(date[2], date[1] as u32, date[0] as u32).single()
    } else {
        None
    }
}

/// Interprets `"24-25.12.2016"` as date.
///
/// Takes care of the old, deprecated, stupid, `dd-dd.mm.yyyy` format, what was I thinking?
/// This is not used in the current format.
pub fn parse_dmy_date_range(date_str:&str) -> Option<Date<Utc>>{
    let date = date_str.split('.')
        .map(|s|s.split('-').nth(0).unwrap_or("0"))
        .map(|f|f.parse().unwrap_or(0))
        .collect::<Vec<i32>>();
    if date[0] > 0 {
        return Some(Utc.ymd(date[2], date[1] as u32, date[0] as u32))
    }
    None
}


/// Gets `Some(Yaml::Hash)` or `None`.
//pub fn get_hash<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a BTreeMap<Yaml,Yaml>> {
pub fn get_hash<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a YamlHash> {
    get(yaml,key).and_then(|y|y.as_hash())
}

/// Gets a `Bool` value.
///
/// **Careful** this is a bit sweeter then ordinary `YAML1.2`,
/// this will interpret `"yes"` and `"no"` as booleans, similar to `YAML1.1`.
/// Actually it will interpret any string but `"yes"` als `false`.
pub fn get_bool(yaml:&Yaml, key:&str) -> Option<bool> {
    get(yaml,key)
        .and_then(|y| y
                  .as_bool()
                  // allowing it to be a str: "yes" or "no"
                  .or_else(|| y.as_str()
                       .map( |yes_or_no|
                             match yes_or_no.to_lowercase().as_ref() // XXX ??? why as_ref?
                             {
                                 "yes" => true,
                                 //"no" => false,
                                 _ => false
                             })
                     ))
}

/// Gets a `Float` value.
///
/// Also takes a `Yaml::I64` and reinterprets it.
pub fn get_f64(yaml:&Yaml, key:&str) -> Option<f64> {
    get(yaml,key).and_then(|y| y.as_f64().or_else(|| y.as_i64().map(|y|y as f64)))
}

/// Gets an `Int` value.
///
/// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::Int`.
pub fn get_int(yaml:&Yaml, key:&str) -> Option<i64> {
    get(yaml,key).and_then(|y|y.as_i64())
}

//TODO this would be nice
//pub fn get_vec_of<T>(yaml:&Yaml, key:&str) -> Option<Vec<T>>{
//    Some(Vec::new())
//}

/// Gets a `&str` value.
///
/// Same mentality as `yaml_rust`, only returns `Some`, if it's a `Yaml::String`.
pub fn get_str<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a str> {
    get(yaml,key).and_then(|y|y.as_str())
}

/// same as `get_str()`, but owned.
pub fn get_string(yaml:&Yaml, key:&str) -> Option<String> {
    get_str(yaml,key).map(ToOwned::to_owned)
}

/// Gets anything **as** `String`.
pub fn get_to_string(yaml:&Yaml, key:&str) -> Option<String> {
    use self::Yaml::*;
    get(yaml,key).and_then(|i| match *i {
        Real(ref inner) | String(ref inner) => Some(inner.to_owned()),
        Boolean(ref inner) => Some(inner.to_string()),
        Integer(ref inner) => Some(inner.to_string()),
        Hash(ref inner) => Some(format!("{:?}", inner)),
        Array(ref inner) => Some(format!("{:?}", inner)),
        _ => None
    })
}

/// Gets a Date in `dd.mm.YYYY` format.
pub fn get_dmy(yaml:&Yaml, key:&str) -> Option<Date<Utc>> {
    get(yaml,key).and_then(|y|y.as_str()).and_then(|d|parse_dmy_date(d))
}

/// Wrapper around `get_path()`.
///
/// Splits path string
/// and replaces `Yaml::Null` and `Yaml::BadValue`.
//#[deprecated(note="use `ProvicdesData` instead")]
pub fn get<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a Yaml>{
    let path = key.split(|c| c == '/' || c == '.')
                  .filter(|k|!k.is_empty())
                  .collect::<Vec<&str>>();
    match get_path(yaml, &path) {
        Some(&Yaml::Null) |
        Some(&Yaml::BadValue) => None,
        content => content
    }
}

/// Returns content at `path` in the yaml document.
//#[deprecated(note="This is old style spec, please use the `ProvidesData` trait")]
fn get_path<'a>(yaml:&'a Yaml, path:&[&str]) -> Option<&'a Yaml>{
    if let Some((&key, remainder)) = path.split_first(){

        return match *yaml{
            Yaml::Hash(ref hash) =>
            {
                if remainder.is_empty(){
                    hash.get(&Yaml::String(key.to_owned()))
                } else {
                    hash.get(&Yaml::String(key.to_owned()))
                        .and_then(|c| get_path(c, remainder))
                }
            },

            Yaml::Array(ref vec) =>
            {
                if let Ok(index) = key.parse::<usize>() {
                    if remainder.is_empty(){
                        vec.get(index)
                    } else {
                        vec.get(index).and_then(|c| get_path(c, remainder))
                    }
                } else { None }
            },
            _ => None
        }

    }
    None
}
