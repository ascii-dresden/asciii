//! Yaml Utility functions

use std::io;
use std::io::Read;
use std::fs::File;
use std::path::Path;

pub use yaml_rust::{Yaml};
use yaml_rust::{YamlLoader};
use yaml_rust::scanner::ScanError;

#[derive(Debug)]
pub enum YamlError{
    Io(io::Error),
    Scan(ScanError)
}

// All you need to make try!() fun again
impl From<io::Error> for YamlError { fn from(ioerror: io::Error)   -> YamlError{ YamlError::Io(ioerror) } }
impl From<ScanError> for YamlError { fn from(scanerror: ScanError) -> YamlError{ YamlError::Scan(scanerror) } }

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

pub fn get_int<'a>(yaml:&'a Yaml, key:&str) -> Option<i64> {
    get(&yaml,&key).and_then(|y|y.as_i64())
}
pub fn get_str<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a str> {
    get(&yaml,&key).and_then(|y|y.as_str())
}

pub fn get<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a Yaml>{
    get_path(&yaml, &key.split('/').filter(|k|!k.is_empty()).collect::<Vec<&str>>())
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
