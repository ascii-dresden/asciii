use std::io;
use std::io::Read;
use std::fs::File;
//use std::fs::Metadata;
use std::path::Path;
use std::path::PathBuf;
use std::env::home_dir;

use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::scanner::ScanError;


const DEFAULT_LOCATION: &'static str = ".ascii-invoicer.yml";

/// Looks for a configuration yaml in your HOME_DIR
pub struct ConfigReader{
    yaml: Vec<Yaml>
}

#[derive(Debug)]
pub enum ConfigError{
    Io(io::Error),
    Scan(ScanError)
}

// All you need to make try!() fun again
impl From<io::Error> for ConfigError {
    fn from(ioerror: io::Error) -> ConfigError{ ConfigError::Io(ioerror) }
}
impl From<ScanError> for ConfigError{
    fn from(scanerror: ScanError) -> ConfigError{ ConfigError::Scan(scanerror) }
}

impl ConfigReader{

    /// The Path of the config file.
    pub fn path() -> PathBuf {
        let home = home_dir().expect("Can't find HOME DIRECTORY");
        return home.join(DEFAULT_LOCATION);
    }

    /// Opens config from `self.path()` and parses Yaml right away.
    pub fn new () -> Result<ConfigReader, ConfigError> {
        let path = ConfigReader::path(); // TODO allow for dynamic paths
        let settings_file = try!(File::open(&path)
            .and_then(|mut file| {
                let mut content = String::new();
                file.read_to_string(&mut content).map(|_| content)
            }));

        Ok(ConfigReader{
            yaml: try!(YamlLoader::load_from_str(&settings_file))
        })
    }

    /// Returns whatever it finds in that position
    pub fn get(&self, key:&str) -> Option<&Yaml>{
        if let Some(first) = self.yaml.get(0){
            return ConfigReader::get_yaml(&first, &key);
        }
        None
    }

    /// Wrapper for `Yaml.get()`
    fn get_yaml<'a>(yaml:&'a Yaml, key:&str) -> Option<&'a Yaml>{
        if let Some(hash) = yaml.as_hash(){
            return hash.get(&Yaml::String(key.into()))
        }
        None
    }

    /// Returns whatever it finds in that position
    ///
    /// Supports simple path syntax: "top/middle/child/node"
    pub fn get_path(&self, path:&str) -> Option<&Yaml>{
        let mut path = Path::new(path).iter().map(|s|s.to_str().unwrap_or("")).collect::<Vec<&str>>();
        ConfigReader::get_path_yaml(&self.yaml[0], &mut path)
    }

    fn get_path_yaml<'a>(yaml:&'a Yaml, path:&mut [&str]) -> Option<&'a Yaml>{
        // recursive: splits off the first element of the path and goes deeper
        if let Some((key, remainder)) = path.split_first_mut(){
            if remainder.is_empty(){
                return ConfigReader::get_yaml(&yaml, key);
            }
            if let Some(content) = ConfigReader::get_yaml(&yaml, key){
                return match content{
                    &Yaml::Hash(_) => ConfigReader::get_path_yaml(content, remainder),
                    _ if remainder.is_empty() =>  Some(content),
                    _ =>  None
                };
            }
        }
        None
    }

    /// Returns the string in the position or an empty string
    pub fn get_str(&self, key:&str) -> &str {
        self.get_path(key).and_then(|y|y.as_str()).unwrap_or("")
    }

    /// Returns the boolean in the position or `false`
    pub fn get_bool(&self, key:&str) -> bool {
        self.get_path(key).and_then(|y|y.as_bool()).unwrap_or(false)
    }

}

#[test]
fn it_works(){
    assert!(ConfigReader::path().exists());
    let config = ConfigReader::new().unwrap();

    assert_eq!(None , config.get("nothing"));

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
