#![allow(dead_code)]
#![allow(unused_imports)]

use std::{io,fs};
use std::path::{Path,PathBuf};

pub fn freeze() {
    let mut _devnull = String::new();
    let _ = io::stdin().read_line(&mut _devnull);
}

pub fn ls(path:&str){
    use std::process::Command;
    println!("tree {}", path);
    let output = Command::new("tree")
        .arg(&path)
        .output()
        .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
    println!("{}", String::from_utf8(output.stdout).unwrap());
}



    //  general
    //    it "knows when to use erb"
    //    it "raises on non existing templates"
    //    it "raises on existing project files"
    //    it "raises on invalid template file formats"
    //    it "raises on invalid project file formats"
    //    it "creates a new file from a static template"
    //    it "creates a new file from an erb template"
    //    it "fills erb from project data"
    //    it "fills erb from default settings"
    //
    //  with no directories
    //    it "notices missing storage directory"
    //    it "notices missing working directory"
    //    it "notices missing archive directory"
    //    it "notices missing templates directory"
    //    it "refuses to create working directory without the storage directory"
    //    it "refuses to create archive directory without the storage directory"
    //    it "refuses to create a new project_folder"
    //    it "creates the storage directory"
    //    it "creates the working directory"
    //    it "creates the archive directory"
    //
    //  with directories
    //    it "checks existing storage directory"
    //    it "checks existing working directory"
    //    it "checks existing archive directory"
    //    it "checks existing templates directory"
    //    it "returns false for missing project"
    //    it "returns path to project folder"
    //    it "finds files in the archive"
    //    it "lists projects"
    //    it "escapes space separated filenames"
    //    it "escapes dash separated filenames"
    //    it "escapes dot separated filenames"
    /*
    /// creates new project_dir and project_file
    /// returns project object
    //pub fn new_project(name: &str, template:&str, data:()){}

    /// produces an Array of @project_class objects
    /// sorted by date (projects must implement date())
    /// if sort is foobar, projects must implement foobar()
    /// output of (foobar must be comparable)
    //fn open_projects(&dir:LuigiDirectory, sort: LuigiSort, year:i32){}

    // internals:
    // def init_logger
    // def init_dirs
    // def check_dir(dir)
    // def check_dirs
    // def load_templates
    // def _new_project_folder(name)
    // def get_project_file_path(name, dir=:working, year=Date.today.year)
    // def open_project_from_path path
    // def map_project_files_working()
    // def map_project_files_archive(year = Date.today.year)
    // def map_project_files(dir = :working, year=Date.today.year)
    // def map_archive_years
    // def list_project_files_working()
    // def list_project_files_archive(year = Date.today.year)
    // def list_project_files_all

    // def sort_projects(projects, sort = :date)
    // def open_projects_all(sort = :date)
    // def open_projects_working(sort = :date)
    // def open_projects_archive(year,sort = :date)
    // def lookup_path_by_name(name, dir = :working, year = Date.today.year)
    // def lookup_by_num(num, dir= :working, sort=:date, year= Date.today.year)
    // def open_project_from_name project_name
    // def get_project_folder( name, dir=:working, year=Date.today.year )
    // def list_project_names(dir = :working, year=Date.today.year)
    // def list_project_files(dir = :working, year=Date.today.year)
    // def filter_by projects, hash
    // def archive_project(project, year = nil, prefix = '')
    // def unarchive_project(project, year = Date.today.year)

    */
