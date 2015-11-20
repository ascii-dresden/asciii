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

fn setup() -> (Config,Luigi){
    let config = ConfigReader::new().unwrap();
    let storage_path = PathBuf::from(config.get_str("path")).join("caterings");
    let storage_path = util::replace_home_tilde(&storage_path);
    let config = Config{
        storage_path : storage_path
    };
    let luigi = Luigi::new(&config.storage_path, "working", "archive", "templates").unwrap();
    (config, luigi)
}

fn assert_existens(storage_path:&Path) {
    assert!(storage_path.exists()
            &&  storage_path.join("working").exists()
            &&  storage_path.join("archive").exists()
            &&  storage_path.join("templates").exists());
}

pub fn show_project(dir:LuigiDir, search_term:&str){
    let (_config,luigi) = setup();

    let projects: Vec<Project> = luigi.list_project_files(dir)
        .iter()
        .map(|path| Project::open(path).unwrap())
        .filter(|project| project.name().to_lowercase().contains(&search_term.to_lowercase()))
        .collect();

    for project in projects{
        println!("{} {} {} {}", project.index(), project.name(), project.manager(), project.date());
    }
}

pub fn list_projects(dir:LuigiDir){
    let (_config,luigi) = setup();
    let project_paths = luigi.list_project_files(dir);
    let projects: Vec<Project> = project_paths.iter().map(|path| Project::open(path).unwrap()).collect();

    for project in projects{
        println!("{} {} {} {}", project.index(), project.name(), project.manager(), project.date());
    }
}
