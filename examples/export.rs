#![allow(dead_code)]
extern crate asciii;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate pretty_assertions;

use asciii::storage::{self,StorageDir};
use asciii::project::Project;
use asciii::project::export::*;

use rustc_serialize::json::ToJson;

fn json_serde(project: &Project) -> String{
    let client: Complete = project.export();
    println!("Serde");
    format!("{:#}", serde_json::to_value(&client).unwrap())
}

fn json_rustc(project: &Project) -> String{
    println!("Rustc Serialize");
    format!("{:#}", project.to_json()["bills"].pretty())
}

fn json_rustc_full(project: &Project) -> String{
    println!("Full Project Json");
    format!("{:#}", project.to_json().pretty())
}

fn compare(project: &Project) {
    let new_export: Complete= project.export();
    let new = format!("{:#}", serde_json::to_value(new_export).unwrap());
    let old = format!("{:#}", project.to_json().pretty());
    assert_eq!(old,new);
    println!("old and new are identical")
}

fn main() {

    let storage = storage::setup().unwrap();
    let projects = storage.open_projects(StorageDir::Archive(2016)).unwrap();
    let project = &projects[2];
    println!("{}\n", json_serde(&project));
    //println!("{}\n", json_rustc(&project));
    //compare(&project);
    //println!("{}\n", json_rustc_full(&project));
}

