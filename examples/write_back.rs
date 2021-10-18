use serde::Serialize;

use asciii::project::export::*;
use asciii::project::spec::*;
use asciii::project::Project;
use asciii::storage::StorageDir;

fn main() {
    let storage = asciii::storage::setup::<Project>().unwrap();
    for project in storage
        .open_projects_dir(StorageDir::Working)
        .unwrap()
        .iter()
        .take(1)
    {
        let name = project.name().unwrap_or("xx");
        let export: Complete = project.export();
        match project.parse_yaml() {
            Ok(_p) => println!("{}", reyaml(&export)),
            Err(e) => println!("cannot parse {} {:#?}", name, e),
        }
    }
}

fn reyaml<T: Serialize>(imp: &T) -> String {
    serde_json::to_string(imp).unwrap()
}
