#![allow(dead_code)]
use std::path::PathBuf;
use std::env::home_dir;
use yaml;
use yaml::{Yaml, YamlError};

const DEFAULT_LOCATION: &'static str = ".ascii-invoicer.yml";

/// Looks for a configuration yaml in your HOME_DIR
// TODO: implement merging of default/personal config
// thinking about that: it would suffice to look in the personal config first, merging is
// unnecessary
pub struct ConfigReader{ yaml: Yaml }

impl ConfigReader{

    /// The Path of the config file.
    pub fn path() -> PathBuf {
        let home = home_dir().expect("Can't find HOME DIRECTORY");
        return home.join(DEFAULT_LOCATION);
    }

    /// Opens config from `self.path()` and parses Yaml right away.
    pub fn new () -> Result<ConfigReader, YamlError> {
        let path = ConfigReader::path();
        Ok(ConfigReader{ yaml: try!(yaml::open_yaml(&path)) })
    }

    /// Returns whatever it finds in that position
    pub fn get(&self, key:&str) -> Option<&Yaml>{
        return yaml::gey_hash_content(&self.yaml, &key);
    }

    /// Returns whatever it finds in that position
    ///
    /// Supports simple path syntax: "top/middle/child/node"
    pub fn get_path(&self, path:&str) -> Option<&Yaml>{
        yaml::get(&self.yaml, &path)
    }

    /// Returns the string in the position or an empty string
    pub fn get_str(&self, key:&str) -> &str {
        yaml::get_str(&self.yaml, &key).unwrap_or("")
    }

    /// Returns the boolean in the position or `false`
    pub fn get_bool(&self, key:&str) -> bool {
        self.get_path(key).and_then(|y|y.as_bool()).unwrap_or(false)
    }

}

#[test]
fn simple_reading(){
    assert!(ConfigReader::path().exists());
    let config = ConfigReader::new().unwrap();

    assert_eq!(config.get("manager_name").unwrap().as_str().unwrap(),
               config.get_str("manager_name"));

    assert_eq!(config.get("colors").unwrap().as_bool().unwrap(),
               config.get_bool("colors"));

    assert!(config.get_path(&"defaults").is_some());
    assert!(config.get_path(&"defaults/includes").is_some());
    assert!(config.get_path(&"defaults/includes/name").is_some());
    assert!(config.get_path(&"defaults/includes/name/foo").is_none());

    println!("{:?}", config.get_path("defaults/includes/name"));
    println!("{:?}", config.get_str("defaults/includes/name"));
    println!("{:?}", config.get_str("defaults/includes/name/foo"));
}
