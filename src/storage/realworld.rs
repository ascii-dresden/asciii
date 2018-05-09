use std::path::{Path,PathBuf};

use super::*;
use super::tests::TestProject;

const STORAGE: &'static str = "/home/hendrik/ascii/caterings";

fn setup() -> (PathBuf, Storage<TestProject>) {
    let storage_path = PathBuf::from(STORAGE);
    let storage = Storage::new(&storage_path, "working", "archive", "templates").unwrap();
    (storage_path, storage)
}

fn assert_existens(storage_path:&Path) {
    assert!(storage_path.exists()
            &&  storage_path.join("working").exists()
            &&  storage_path.join("archive").exists()
            &&  storage_path.join("templates").exists());
}

#[test]
#[ignore]
fn list_template_files(){
    let (storage_path, storage) = setup();
    //storage.create_dirs().unwrap();
    assert_existens(&storage_path);

    let templates = storage.list_template_files().unwrap();
    println!("{:#?}",templates);
    assert!(templates.len() == 3);
}

#[test]
#[ignore]
fn list_archives(){
    let (_storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok());

    let mut archives = storage.list_archives().unwrap();
    archives.sort();
    println!("ARCHIVES\n{:#?}", archives);

    assert!(archives[0].ends_with("2012"));
    assert!(archives[1].ends_with("2013"));
    assert!(archives[2].ends_with("2014"));
    assert!(archives[3].ends_with("2015"));
}

#[test]
#[ignore]
fn list_project_folders(){
    let (_storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok());

    //let projects = storage.list_project_files(StorageDir::Archive(2015));
    let projects = storage.list_project_files(StorageDir::Working);
    println!("Projects");
    for p in projects{
        println!("{:#?}", p);
    }
}

