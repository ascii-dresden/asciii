extern crate asciii;

use std::error::Error;
use asciii::storage::{Storable,StorageDir};

fn main() {

    let dir = StorageDir::All;

    let luigi = asciii::setup_storage().unwrap();
    for project in luigi.open_working_dir_projects().unwrap() {
        println!("{:#?}", project);
    }
}
