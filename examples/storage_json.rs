#![allow(dead_code)]
extern crate asciii;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;

use asciii::storage;
use asciii::storage::Storable;
use asciii::project::Project;

fn main() {
    let storage = storage::setup::<Project>().unwrap();
    let serde_string = serde_json::to_value(&storage.paths())
                                    .unwrap();
    println!("{:#?}", serde_string);
}

