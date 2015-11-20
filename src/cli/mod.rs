use std::path::{Path,PathBuf};
use config::ConfigReader;
use manager::{Luigi,LuigiDir};
use manager::LuigiProject;
use project::Project;
use util;

#[derive(Debug)]
struct Config{
    pub storage_path: PathBuf
}

fn setup() -> Config{
    let config = ConfigReader::new().unwrap();
    let storage_path = PathBuf::from(config.get_str("path")).join("caterings");
    let storage_path = util::replace_home_tilde(&storage_path);
    Config{
        storage_path : storage_path
    }
}

fn assert_existens(storage_path:&Path) {
    assert!(storage_path.exists()
            &&  storage_path.join("working").exists()
            &&  storage_path.join("archive").exists()
            &&  storage_path.join("templates").exists());
}

pub fn list_projects(dir:LuigiDir){
    let config = setup();
    //TODO working, archive and templates come from a defeault config
    let luigi = Luigi::new(&config.storage_path, "working", "archive", "templates").unwrap();
    let project_paths = luigi.list_project_files(dir);
    let projects : Vec<Project> = project_paths.iter().map(|path| Project::open(path).unwrap()).collect();

    for project in projects{
        println!("{} {} {} {}", project.index(), project.name(), project.manager(), project.date());
    }
}
