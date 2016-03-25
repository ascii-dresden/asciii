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
use util;
use repo::{GitStatus,Repository};

pub mod print;

// TODO keep this up to date or find a way to make this dynamic
const STATUS_ROWS_WIDTH:u16 = 96;


fn setup_luigi_with_git() -> Luigi {
    execute(||Luigi::new_with_git(util::get_storage_path(), "working", "archive", "templates"))
}

fn setup_luigi() -> Luigi {
    execute(|| Luigi::new(util::get_storage_path(), "working", "archive", "templates"))
}

/// Execute a command returning a LuigiError
/// TODO make this a `try!` like macro
fn execute<F, S>(command:F) -> S where F: FnOnce() -> LuigiResult<S> {
    match command(){
        Ok(s) => s,
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
    let luigi = setup_luigi_with_git();
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
        print::print_projects(print::status_rows(&projects,&luigi.repository.unwrap()));
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

/// Command LIST --years
pub fn list_years(){
    let luigi = setup_luigi();
    let years = execute(||luigi.list_years());
    println!("{:?}", years);
}




/// Command LIST --years
pub fn list_paths(dir:LuigiDir){
    let luigi = setup_luigi();
    let paths = execute(||luigi.list_project_files(dir));
    for path in paths{
    println!("{}", path.display());
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

    if paths.is_empty(){
        println!{"Nothing found for {:?}", search_term}
    }
    else {
        util::open_in_editor(&editor, paths);
    }
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
    //let project = execute(|| luigi.create_project::<Project>(&project_name, &template_name));
    let project = luigi.create_project::<Project>(&project_name, &template_name).unwrap();
    let project_file = project.file();
    if edit { util::open_in_editor(&editor, vec![project_file.display().to_string()]);
    }
}


/// Command ARCHIVE <NAME>
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
            luigi.unarchive_project_file(&projects[0]).unwrap();
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
    //println!("{:?}{:?}", &editor, vec![CONFIG.path.to_str().unwrap().to_owned()]);
    util::open_in_editor(&editor, vec![CONFIG.path.to_str().unwrap().to_owned()]);
}

/// Command CONFIG --default
pub fn config_show_default(){
    println!("{}", config::DEFAULT_CONFIG);
}


/// Command STATUS
pub fn git_status(){
    let luigi = setup_luigi_with_git();
    println!("{:#?}", luigi);
    println!("{:#?}", luigi.repository.unwrap().statuses);
}

/// Command REMOTE
/// exact replica of `git remote -v`
pub fn git_remote(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap().repo;

    for remote_name in repo.remotes().unwrap().iter(){ // Option<Option<&str>> oh, boy
        if let Some(name) = remote_name{
            if let Ok(remote) = repo.find_remote(name){
            println!("{}  {} (fetch)\n{}  {} (push)",
                    remote.name().unwrap_or("no name"),
                    remote.url().unwrap_or("no url"),
                    remote.name().unwrap_or("no name"),
                    remote.pushurl().or(remote.url()).unwrap_or(""),
                    );
            }else{println!("no remote")}
        }else{println!("no remote name")}
    }
}

/// Command PULL
pub fn git_pull(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    repo.pull();
}
