extern crate asciii;
extern crate serde;
extern crate serde_yaml;
extern crate serde_json;
extern crate toml;

use serde::Serialize;

use asciii::project::Project;
use asciii::storage::StorageDir;
use asciii::project::import;
use asciii::project::export::*;
use asciii::project::spec::*;

fn main() {

    let storage = asciii::storage::setup::<Project>().unwrap();
    for project in storage.open_projects_dir(StorageDir::Working).unwrap().iter().take(1) {
        let name = project.name().unwrap_or("xx");
        let export: Complete = project.export();
        match project.parse_yaml() {
            Ok(p)  => println!("{}", retoml(&export)),
            Err(e) => println!("cannot parse {} {:#?}", name, e),
        }
    }
}

fn reyaml(imp: &import::Project) -> String {
    serde_yaml::to_string(imp).unwrap()

}

fn retoml<T: Serialize>(imp: &T) -> String {
    serde_json::to_string(imp).unwrap()

}

