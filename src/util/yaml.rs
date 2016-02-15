//! Yaml Utility functions

use std::fmt;
use std::io;
use std::io::Read;
use std::fs::File;
use std::path::Path;

pub use yaml_rust::{Yaml};
use yaml_rust::{YamlLoader};
use yaml_rust::scanner::ScanError;
use chrono::*;

#[derive(Debug)]
pub enum YamlError{
    Io(io::Error),
    Scan(ScanError)
}

// All you need to make try!() fun again
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

//TODO Rename to open()
pub fn open_yaml( path:&Path ) -> Result<Yaml, YamlError> {
    let file_content = try!(File::open(&path)
                             .and_then(|mut file| {
                                 let mut content = String::new();
                                 file.read_to_string(&mut content).map(|_| content)
                             }));
    parse( &file_content )
}

pub fn parse( file_content:&str ) -> Result<Yaml, YamlError> {
    Ok(try!(YamlLoader::load_from_str(&file_content)).get(0).unwrap().to_owned())
}

pub fn parse_fwd_date(date_str:&str) -> Option<Date<UTC>>{
    let date = date_str.split('.')
        .map(|f|f.parse().unwrap_or(0))
        .collect::<Vec<i32>>();
    if date[0] > 0 {
        // XXX this neglects the old "01-05.12.2015" format
        return Some(UTC.ymd(date[2], date[1] as u32, date[0] as u32))
    }
    None
}

//takes care of the old, stupid, dd-dd.mm.yyyy format, what was I thinking?
pub fn parse_fwd_date_range(date_str:&str) -> Option<Date<UTC>>{
    let date = date_str.split('.')
        .map(|s|s.split('-').nth(0).unwrap_or("0"))
        .map(|f|f.parse().unwrap_or(0))
        .collect::<Vec<i32>>();
    if date[0] > 0 {
        return Some(UTC.ymd(date[2], date[1] as u32, date[0] as u32))
    }
    None
}


use std::collections::BTreeMap;
pub fn get_hash<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a BTreeMap<Yaml,Yaml>> {
    get(&yaml,&key).and_then(|y|y.as_hash())
}

pub fn get_bool<'a>(yaml:&'a Yaml, key:&str) -> Option<bool> {
    get(&yaml,&key)
        .and_then(|y| y
                  .as_bool()
                  // allowing it to be a str: "yes" or "no"
                  .or( y.as_str()
                       .map( |yes_or_no|
                             match yes_or_no.to_lowercase().as_ref() // XXX ??? why as_ref?
                             {
                                 "yes" => true,
                                 "no" => false,
                                 _ => false
                             })
                     ))
}

// also takes a Yaml::I64 and reinterprets it
pub fn get_f64<'a>(yaml:&'a Yaml, key:&str) -> Option<f64> {
    get(&yaml,&key).and_then(|y| y.as_f64().or( y.as_i64().map(|y|y as f64)))
}

pub fn get_int<'a>(yaml:&'a Yaml, key:&str) -> Option<i64> {
    get(&yaml,&key).and_then(|y|y.as_i64())
}

pub fn get_str<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a str> {
    get(&yaml,&key).and_then(|y|y.as_str())
}

pub fn get_string(yaml:&Yaml, key:&str) -> Option<String> {
    get_str(&yaml,&key).map(|s|s.to_owned())
}

pub fn get_dmy<'a>(yaml:&'a Yaml, key:&str) -> Option<Date<UTC>> {
    get(&yaml,&key).and_then(|y|y.as_str()).and_then(|d|parse_fwd_date(d))
}

pub fn get<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a Yaml>{
    match get_path(&yaml, &key.split('/').filter(|k|!k.is_empty()).collect::<Vec<&str>>()) {
        Some(&Yaml::Null) => None,
        content => content
    }
}

fn get_path<'a>(yaml:&'a Yaml, path:&[&str]) -> Option<&'a Yaml>{
    if let Some((&key, remainder)) = path.split_first(){

        return match yaml{
            &Yaml::Hash(ref hash) =>
            {
                if remainder.is_empty(){
                    hash.get(&Yaml::String(key.to_owned()))
                } else {
                    hash.get(&Yaml::String(key.to_owned()))
                        .and_then(|c| get_path(c, remainder))
                }
            },

            &Yaml::Array(ref vec) =>
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
