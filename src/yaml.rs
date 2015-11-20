//! Yaml Utility functions

use std::io;
use std::io::Read;
use std::fs::File;
use std::path::Path;

pub use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::scanner::ScanError;

#[derive(Debug)]
pub enum YamlError{
    Io(io::Error),
    Scan(ScanError)
}

// All you need to make try!() fun again
impl From<io::Error> for YamlError {
    fn from(ioerror: io::Error) -> YamlError{ YamlError::Io(ioerror) }
}
impl From<ScanError> for YamlError{
    fn from(scanerror: ScanError) -> YamlError{ YamlError::Scan(scanerror) }
}

pub fn open_yaml( path:&Path ) -> Result<Yaml, YamlError> {
    let settings_file = try!(File::open(&path)
                             .and_then(|mut file| {
                                 let mut content = String::new();
                                 file.read_to_string(&mut content).map(|_| content)
                             }));

    Ok(try!(YamlLoader::load_from_str(&settings_file)).get(0).unwrap().to_owned())
}

/// Wrapper for `Yaml.get()`
pub fn gey_hash_content<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a Yaml>{
    if let Some(hash) = yaml.as_hash(){
        return hash.get(&Yaml::String(key.into()))
    }
    None
}

fn get_path<'a>(yaml:&'a Yaml, path:&mut [&str]) -> Option<&'a Yaml>{
    // recursive: splits off the first element of the path and goes deeper
    if let Some((key, remainder)) = path.split_first_mut(){
        if remainder.is_empty(){
            return gey_hash_content(&yaml, key);
        }
        if let Some(content) = gey_hash_content(&yaml, key){
            return match content{
                &Yaml::Hash(_) => get_path(content, remainder),
                _ if remainder.is_empty() =>  Some(content),
                _ =>  None
            };
        }
    }
    None
}

pub fn get<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a Yaml>{
    let mut path = Path::new(key).iter().map(|s|s.to_str().unwrap_or("")).collect::<Vec<&str>>();
    get_path(&yaml, &mut path)
}
pub fn get_int<'a>(yaml:&'a Yaml, key:&str) -> Option<i64> {
    let mut path = Path::new(key).iter().map(|s|s.to_str().unwrap_or("")).collect::<Vec<&str>>();
    get_path(&yaml,&mut path).and_then(|y|y.as_i64())
}
pub fn get_str<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a str> {
    let mut path = Path::new(key).iter().map(|s|s.to_str().unwrap_or("")).collect::<Vec<&str>>();
    get_path(&yaml,&mut path).and_then(|y|y.as_str())
}

