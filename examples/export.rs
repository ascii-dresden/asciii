extern crate asciii;
extern crate serde_json;

use asciii::CONFIG;
use asciii::storage::{self,Storage,StorageDir};
use asciii::project::Project;
use asciii::project::export::*;

fn with_project(project:Project) {
    let client: Client = project.export();
    println!("{:#?}", client);

    let client = serde_json::to_string(&client).unwrap();
    println!("{}", client);
    println!("--------------------");
}

fn main() {

    let dir = StorageDir::All;

    let storage = storage::setup().unwrap();
    storage.simple_with_projects(dir, None, with_project);

}

