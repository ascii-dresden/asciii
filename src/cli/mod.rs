//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

#![allow(unused_variables)]
use std::path::{Path,PathBuf};
use std::ffi::OsStr;

use chrono::UTC;

use config;
use super::CONFIG;
use manager::{Luigi,LuigiDir};
use manager::LuigiProject;
use project::Project;
use repo::Repo;
use util;

pub mod print;

fn setup_luigi() -> Luigi{
    let storage_path = PathBuf::from(CONFIG.get_str("path")).join("caterings");
    let storage_path = util::replace_home_tilde(&storage_path);
    let luigi = Luigi::new(&storage_path, "working", "archive", "templates").unwrap();
    luigi
}

fn assert_existens(storage_path:&Path) {
    assert!(storage_path.exists()
            &&  storage_path.join("working").exists()
            &&  storage_path.join("archive").exists()
            &&  storage_path.join("templates").exists());
}

fn status(){
    let luigi = setup_luigi();
    let repo = Repo::new(luigi.storage_dir()).unwrap();

    let project_paths = luigi.list_project_files(LuigiDir::Working);
    let projects: Vec<Project> = project_paths
        .iter()
        .map(|path| Project::open(path).unwrap())
        .collect();

    println!("{:#?}", repo.status);
    print::print_projects(print::status_rows(&projects,&repo));
}

pub enum SortOptions{ Index }

fn sort_by_(option:&str, projects:&mut [Project]){
    println!("sorting by {}", option);
    match option {
        "manager" => sort_by_manager(projects),
        "date" => sort_by_date(projects),
        "name" => sort_by_name(projects),

        _ => sort_by_index(projects),
    }
}

fn sort_by_index(projects:&mut [Project]){
    projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned())))
}

fn sort_by_name(projects:&mut [Project]){
    projects.sort_by(|pa,pb| pa.name().cmp( &pb.name()))
}

fn sort_by_date(projects:&mut [Project]){
    projects.sort_by(|pa,pb| pa.date().cmp( &pb.date()))
}

fn sort_by_manager(projects:&mut [Project]){
    projects.sort_by(|pa,pb| pa.manager().cmp( &pb.manager()))
}

/// Opens up all projects to look inside and check content.
///
/// TODO This could be parallelized
/// TODO move this into `Luigi`
pub fn search_projects(dir:LuigiDir, search_term:&str) -> Vec<Project> {
    let luigi = setup_luigi();

    let projects: Vec<Project> = luigi.list_project_files(dir)
        .iter()
        .map(|path| Project::open(path).unwrap())
        .filter(|project| project.name().to_lowercase().contains(&search_term.to_lowercase()))
        .collect();

    projects
}

/// Command LIST [--archive, --all]
pub fn list_projects(dir:LuigiDir, sort:&str){

    let luigi = setup_luigi();
    let project_paths = luigi.list_project_files(dir);
    let mut projects: Vec<Project> = project_paths.iter()
        .filter_map(|path| Project::open(path).ok())
        .collect();

    sort_by_(sort, &mut projects);
    let repo = Repo::new(luigi.storage_dir()).unwrap();
    print::print_projects(print::status_rows(&projects,&repo));
}

/// Command LIST --broken
pub fn list_broken_projects(dir:LuigiDir){
    let luigi = setup_luigi();
    let projects: Vec<Project> = luigi.list_broken_projects(dir)
        .iter()
        .map(|p|Project::open(p).unwrap()).collect() ;
    print::print_projects(print::simple_rows(&projects));
}

/// Command LIST --templates
pub fn list_templates(){
    let luigi = setup_luigi();
    let template_paths = luigi.list_template_files();

    for path in template_paths{
        println!("{}", path.display());
    }
}

/// Command EDIT
pub fn edit_project(dir:LuigiDir, search_term:&str, editor:&str){
    let paths = search_projects(dir, &search_term)
        .iter()
        .filter_map(|project|
                    project.file().to_str()
                    .map(|s|s.to_owned())
                    )
        .collect::<Vec<String>>();

    util::open_in_editor(&editor, paths);
}

pub fn edit_template(name:&str, editor:&str){
    let luigi = setup_luigi();
    let template_paths = luigi.list_template_files()
        .iter()
        .filter(|f|f
                .file_stem()
                .unwrap_or(&OsStr::new("")) == name)
        .map(|p|p.display().to_string())
        .collect::<Vec<String>>();
    util::open_in_editor(&editor, template_paths);
}

/// Command SHOW
pub fn show_project(dir:LuigiDir, search_term:&str){
    for project in search_projects(dir, &search_term){
        println!("{} {} {} {}, {}",
                 project.index().unwrap_or("".into()),
                 project.name(),
                 project.manager(),
                 project.date().unwrap(),
                 project.file().display()
                 );
    }
}

/// Command SHOW --template
use templater::Templater;
pub fn show_template(name:&str){
    let luigi = setup_luigi();
    let templater = Templater::new(&luigi.get_template_file(name).unwrap()).unwrap();
    println!("{:#?}", templater.list_keywords());
}

/// Command NEW
pub fn new_project(project_name:&str, template_name:&str, editor:&str, edit:bool){
    let luigi = setup_luigi();
    let project = luigi.create_project::<Project>(&project_name, &template_name).unwrap();
    let project_file = project.file();
    let project_path = project_file.parent().unwrap();
    util::open_in_editor(&editor, vec![project_path.display().to_string()]);

    println!("created?");
}

/// Command CONFIG --show
pub fn config_show(path:&str){
    //TODO config_show could be prettier
    println!("{:#?}", CONFIG.get_str(&path));
}

/// Command CONFIG --edit
pub fn config_edit(editor:&str){
    util::open_in_editor(&editor, vec![CONFIG.path.to_str().unwrap().to_owned()]);
}

/// Command CONFIG --default
pub fn config_show_default(){
    println!("{}", config::DEFAULT_CONFIG);
}
