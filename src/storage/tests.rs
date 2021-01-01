use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

use chrono::prelude::*;
use tempdir::TempDir;
use maplit::hashmap;

use crate::util;
use super::*;

// TODO: add tests for file or directories in return values

#[allow(dead_code)]
pub struct TestProject {
    file_path: PathBuf,
}

impl Storable for TestProject{
    // creates in tempfile
    fn from_template(project_name: &str, template: &Path, _fill: &HashMap<&str, String>) -> Result<StorableAndTempDir<Self>, Error> where Self: Sized {
        // generates a temp file
        let temp_dir  = TempDir::new_in("./target/debug/build/",&project_name).unwrap();
        let temp_file = temp_dir.path().join(project_name);

        // just copy over template
        fs::copy(template, &temp_file)?;

        // project now lives in the temp_file
        let project = TestProject {
            file_path: temp_file,
        };

        Ok(StorableAndTempDir {
            storable: project,
            temp_dir
        })
    }

    fn short_desc(&self) -> String{ self.file().file_stem().unwrap().to_str().unwrap().to_owned() }
    fn modified_date(&self) -> Option<Date<Utc>>{ Some(Utc::today()) }
    fn file(&self) -> PathBuf{ self.file_path.to_owned() }
    fn set_file(&mut self, new_file:&Path){ self.file_path = new_file.to_owned(); }
    fn index(&self) -> Option<String>{ Some("ZZ99".into()) }
    fn prefix(&self) -> Option<String>{ self.index() }

    fn open_folder(path:&Path) -> Result<Self, Error>{
        Self::open_file(path)
    }

    fn open_file(path:&Path) -> Result<Self, Error>{
        Ok(TestProject{
            file_path: PathBuf::from(path)
        })
    }
    fn matches_filter(&self, _key: &str, _val: &str) -> bool {false}
    fn matches_search(&self, _term: &str) -> bool {false}
    fn is_ready_for_archive(&self) -> bool {true}

}


// TODO: implement failing cases
const TEST_PROJECTS:[&str; 4] = [
    "test1", "test2",
    "foobar", "ich schreibe viel zu längliche projektnamen!",
];


fn setup() -> (TempDir, PathBuf, Storage<TestProject>) {
    let dir = TempDir::new_in(Path::new("."),"storage_test").unwrap();
    let storage_path = dir.path().join("storage_test");
    let storage = Storage::try_new(&storage_path, "working", "archive", "templates").unwrap();
    (dir, storage_path, storage)
}

fn assert_existence(storage_path:&Path) {
    assert!(storage_path.exists()
            &&  storage_path.join("working").exists()
            &&  storage_path.join("archive").exists()
            &&  storage_path.join("templates").exists());
}

fn copy_template(target:PathBuf) {
    fs::copy("./templates/default.tyml", target.join("template1.tyml")).unwrap();
    fs::copy("./templates/default.tyml", target.join("template2.tyml")).unwrap();
}

#[test]
fn create_dirs() {
    let (dir , storage_path, storage) = setup();
    storage.create_dirs().unwrap();
    assert_existence(&storage_path);

    // calling it again does not cause problems
    assert!(storage.create_dirs().is_ok());
    assert_existence(&storage_path);

    util::ls(&dir.path().to_string_lossy());
}

#[test]
fn list_template_files(){
    let (_dir , storage_path, storage) = setup();
    storage.create_dirs().unwrap();
    assert_existence(&storage_path);

    copy_template(storage_path.join("templates"));

    let templates = storage.list_template_files().unwrap();
    println!("{:#?}",templates);
    assert!(templates.len() == 2);

    ////util::freeze();
}

#[test]
fn create_archive(){
    let (_dir , storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok());
    assert_existence(&storage_path);
    storage.create_archive(2001).unwrap();
    storage.create_archive(2002).unwrap();
    storage.create_archive(2002).unwrap(); // should this fail?
    assert!(storage_path.join("archive").join("2001").exists());
    assert!(storage_path.join("archive").join("2002").exists());
    util::ls(&_dir.path().to_string_lossy());
}

#[test]
fn list_archives(){
    let (_dir , storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok());
    assert_existence(&storage_path);
    storage.create_archive(2001).unwrap();
    storage.create_archive(2002).unwrap();
    storage.create_archive(1999).unwrap();
    util::ls(&_dir.path().to_string_lossy());

    let mut archives = storage.list_archives().unwrap();
    let mut years = storage.list_years().unwrap();
    archives.sort();
    years.sort();
    println!("ARCHIVES\n{:#?}", archives);

    assert!(archives[0].ends_with("1999"));
    assert!(archives[0].is_dir());
    assert!(archives[1].ends_with("2001"));
    assert!(archives[1].is_dir());
    assert!(archives[2].ends_with("2002"));
    assert!(archives[2].is_dir());

    println!("ARCHIVES\n{:#?}", years);
    assert_eq!(years[0], 1999);
    assert_eq!(years[1], 2001);
    assert_eq!(years[2], 2002);
}

#[test]
fn create_project(){
    let (_dir , storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok());
    assert_existence(&storage_path);
    copy_template(storage_path.join("templates"));

    let templates = storage.list_template_names().unwrap();

    for test_project in TEST_PROJECTS.iter() {
        let project     = storage.create_project(&test_project, &templates[0], &hashmap!()).unwrap();
        let target_file = project.file();
        let target_path = target_file.parent().unwrap();
        assert!(target_path.exists());
        assert!(target_file.exists());
        util::ls(&target_path.display().to_string());
        assert_eq!(target_file, storage.get_project_file(&target_path).unwrap());

        let project_dir = storage.get_project_dir(test_project, StorageDir::Working);
        assert!(project_dir.unwrap().exists());

        let project_dir = storage.get_project_dir(test_project, StorageDir::Working);
        assert_eq!(project_dir.unwrap(), target_path);
    }
}

#[test]
fn archive_project_by_name(){
    let (_dir , storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok());
    assert_existence(&storage_path);
    copy_template(storage_path.join("templates"));

    let templates = storage.list_template_names().unwrap();
    log::trace!("templates: {:#?}", templates);
    for test_project in TEST_PROJECTS.iter() {
        // tested above
        let origin = storage.create_project( &test_project, &templates[0], &hashmap!{}).unwrap();

        // the actual tests
        assert!(storage.archive_project_by_name(&test_project, 2015, None).is_ok());
        assert!(!origin.file().exists());

        assert!(storage.get_project_dir(&test_project, StorageDir::Working).is_err());
        assert!(storage.get_project_dir(&test_project, StorageDir::Archive(2015)).is_ok());

        //let false_origin = storage.create_project(&test_project, &templates[0]).unwrap();
        assert!(storage.archive_project_by_name(&test_project, 2015, None).is_err());
    }
}

#[test]
fn archive_project(){
    let (_dir , storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok(), "could not even create storage in {:?}", storage_path);
    assert_existence(&storage_path);
    copy_template(storage_path.join("templates"));

    let year = Utc::today().year();

    let templates = storage.list_template_names().unwrap();
    for test_project_name in TEST_PROJECTS.iter() {
        // tested above
        let project = storage.create_project( &test_project_name, &templates[0], &hashmap!{}).unwrap();

        // Before archiving
        assert!(project.file().exists());
        assert!(storage.get_project_dir(&test_project_name, StorageDir::Working).is_ok());

        // ARCHIVING
        assert!(storage.archive_project(&project, project.year().unwrap()).is_ok());

        // After archiving
        assert!(!project.file().exists());
        assert!(storage.get_project_dir(&test_project_name, StorageDir::Working).is_err());
        assert!(storage.get_project_dir(&test_project_name, StorageDir::Archive(year)).is_ok());

        assert!(storage.archive_project(&project, year).is_err());
    }
}

#[test]
fn unarchive_project_dir(){
    let (_dir , storage_path, storage) = setup();
    assert!(storage.create_dirs().is_ok());
    assert_existence(&storage_path);
    copy_template(storage_path.join("templates"));

    let templates = storage.list_template_names().unwrap();
    for test_project in TEST_PROJECTS.iter() {
        let _origin = storage.create_project( &test_project, &templates[0], &hashmap!{}).unwrap();
        storage.archive_project_by_name(test_project, 2015, None).unwrap();
    }

    for year in storage.list_years().unwrap(){
        println!("{:?}", year);
        for proj in storage.list_project_folders(StorageDir::Archive(year)).unwrap() {
            assert!(storage.unarchive_project_dir(&proj).is_ok());
            assert!(storage.unarchive_project_dir(&proj).is_err());
        }
    }
}
