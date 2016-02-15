//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::path::{Path,PathBuf};
use std::ffi::OsStr;
use std::process::exit;

use chrono::UTC;
use terminal_size::{Width, Height, terminal_size };

use config;
use super::CONFIG;
use manager::{Luigi, LuigiDir, LuigiProject, LuigiResult, LuigiError};
use project::Project;
use repo::Repo;
use util;

pub mod print;

// TODO keep this up to date or find a way to make this dynamic
const STATUS_ROWS_WIDTH:u16 = 90;

fn setup_luigi() -> Luigi{
    let storage_path = PathBuf::from( CONFIG.get_str("path")) .join( CONFIG.get_str("dirs/storage"));
    let storage_path = util::replace_home_tilde(&storage_path);
    let luigi = Luigi::new(&storage_path, "working", "archive", "templates").unwrap();
    luigi
}

/// Execute a command returning a LuigiError
/// TODO make this a `try!` like macro
fn execute<F, S>(command:F) -> S where F: FnOnce() -> LuigiResult<S> {
    match command(){
        Ok(s) => return s,
        Err(lerr) => { println!("ERROR: {:?}", lerr); exit(1) }
    }
}

pub enum SortOptions{ Index }

fn sort_by_option(option:&str, projects:&mut [Project]){
    match option {
        "manager" => sort_by_manager(projects),
        "date"    => sort_by_date(projects),
        "name"    => sort_by_name(projects),

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




/// Command LIST [--archive, --all]
pub fn list_projects(dir:LuigiDir, sort:&str, simple:bool){
    let luigi = setup_luigi();
    let project_paths = execute(||luigi.list_project_files(dir));
    let mut projects: Vec<Project> = project_paths.iter()
        .filter_map(|path| match Project::open(path){
            Ok(project) => Some(project),
            Err(err) => {
                println!("Erroneous Project: {}\n {}", path.display(), err);
                None
            }
        })
        .collect();

    sort_by_option(sort, &mut projects);

    let wide_enough = match terminal_size() {
        Some((Width(w), _)) if w >= STATUS_ROWS_WIDTH => true,
        _ => false
    };

    if simple || !wide_enough {
        print::print_projects(print::simple_rows(&projects));
    }
    else {
        let repo = Repo::new(luigi.storage_dir()).unwrap();
        print::print_projects(print::status_rows(&projects,&repo));
    }
}

/// Command LIST --broken
pub fn list_broken_projects(dir:LuigiDir){
    use util::yaml::YamlError;
    let luigi = setup_luigi();
    let invalid_files = execute(||luigi.list_project_files(dir));
    let tups = invalid_files
        .iter()
        .filter_map(|dir| Project::open(dir).err().map(|e|(e, dir)) )
        .collect::<Vec<(YamlError,&PathBuf)>>();

    for (err,path) in tups{
        println!("{}: {}", path.display(), err);
    }
}

/// Command LIST --templates
pub fn list_templates(){
    let luigi = setup_luigi();
    let template_paths = execute(||luigi.list_template_files());

    for path in template_paths{
        if let Some(stem) = path.file_stem(){
            println!("{}", stem.to_string_lossy());
        } else {
            println!("broken template: {}", path.display());
        }
    }
}




/// Command EDIT
pub fn edit_project(dir:LuigiDir, search_term:&str, editor:&str){
    let luigi = setup_luigi();
    let paths = execute(||luigi.search_projects(dir, &search_term))
        .iter()
        .filter_map(|path| Project::open(path).ok())
        .filter_map(|project|
                    project.file().to_str()
                    .map(|s|s.to_owned())
                    )
        .collect::<Vec<String>>();

    util::open_in_editor(&editor, paths);
}

/// Command EDIT --template
pub fn edit_template(name:&str, editor:&str){
    let luigi = setup_luigi();
    let template_paths = execute(||luigi.list_template_files())
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
    let luigi = setup_luigi();
    for project in execute(||luigi.search_projects(dir, &search_term))
        .iter()
        .filter_map(|path| Project::open(path).ok())
    {
        print::show_items(&project);
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
    let project = execute(|| luigi.create_project::<Project>(&project_name, &template_name));
    let project_file = project.file();
    if edit { util::open_in_editor(&editor, vec![project_file.display().to_string()]); }
}


/// Command ARCHIVW <NAME>
pub fn archive_project(name:&str, manual_year:Option<i32>){
    let luigi = setup_luigi();
    if let Ok(projects) = luigi.search_projects(LuigiDir::Working, name){
        for project in projects.iter().filter_map(|path| Project::open(path).ok()) {
            if project.valid_stage3().is_ok(){
                let year = manual_year.or(project.year()).unwrap();
                println!("archiving {} ({})",  project.ident(), project.year().unwrap());
                execute(||luigi.archive_project(&project, year));
            }
            else {
                println!("CANNOT archive {} ({})",
                project.ident(), project.file().display());
            }
        }
    }
}


/// Command UNARCHIVW <YEAR> <NAME>
pub fn unarchive_project(year:i32, name:&str){
    let luigi = setup_luigi();
    if let Ok(projects) = luigi.search_projects(LuigiDir::Archive(year), name){
        if projects.len() == 1 {
            println!("{:?}", projects);
            luigi.unarchive_project_file(&projects[0]);
        } else{
            println!("Ambiguous: multiple matches: {:#?}", projects );
        }
    }
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


/// Command STATUS
pub fn git_status(){
    println!("git_status");
    let luigi = setup_luigi();
    let repo = Repo::new(luigi.storage_dir()).unwrap();

    let project_paths = execute(||luigi.list_project_files(LuigiDir::Working));
    let projects: Vec<Project> = project_paths
        .iter()
        .filter_map(|path| Project::open(path).ok())
        .collect();

    println!("{:#?}", repo.status);
}

/// Command PULL
pub fn git_pull(){
    println!("git_pull");

    let luigi = setup_luigi();
    let repo = Repo::new(luigi.storage_dir()).unwrap();
    repo.fetch();
}
