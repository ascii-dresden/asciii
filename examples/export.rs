#![allow(dead_code)]
extern crate asciii;
extern crate serde;
extern crate serde_json;

use asciii::storage::{self,StorageDir};
use asciii::project::Project;
use asciii::project::export::*;


fn json_serde(project: &Project) -> String {
    let client: Complete = project.export();
    println!("Serde");
    format!("{:#}", serde_json::to_value(&client).unwrap())
}

fn main() {
    let storage = storage::setup().unwrap();
    let projects = storage.open_projects(StorageDir::Archive(2016)).unwrap();
    let project = &projects[2];
    println!("{}\n", json_serde(&project));
}

