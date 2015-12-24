//! Hooks for the commandline interface
//!
//! # Note to self
//! Put as little logic in here as possible.
//! That makes it easier to derive a pure library version later.

use std::path::{Path,PathBuf};
use std::ffi::OsStr;
use std::process::exit;

use chrono::UTC;
use clap::{App, SubCommand, Arg};

use config;
use super::CONFIG;
use manager::{Luigi, LuigiDir, LuigiProject, LuigiResult, LuigiError};
use project::Project;
use repo::Repo;
use util;

pub mod print;

pub fn app(){
    let matches = App::new("ascii-invoicer")
        .version(&crate_version!()[..])
        .author("Hendrik Sollich <hendrik@hoodie.de>")
        .about("The ascii invoicer III")
        .arg_required_else_help(true)

        .subcommand(SubCommand::with_name("list")

                    .arg( Arg::with_name("archive")
                          .help("list archived projects")
                          .short("a").long("archive")
                          .takes_value(true))

                    .arg( Arg::with_name("sort")
                          .help("sort by [date | index | name | manager]")
                          .short("s").long("sort")
                          //.possible_values(vec![ String::from("date"), String::from("index"), String::from("name"), String::from("manager") ])
                          .takes_value(true))

                    .arg( Arg::with_name("all")
                          .help("List all projects, ever")
                          .long("all"))

                    .arg( Arg::with_name("templates")
                          .help("list templates")
                          .short("t").long("templates"))

                    .arg( Arg::with_name("broken")
                          .help("list broken projects (without project file)")
                          .short("b").long("broken"))

                    )

        .subcommand(SubCommand::with_name("edit")
                    .about("Edit a specific project")

                    .arg( Arg::with_name("search_term")
                          .help("Search term, possibly event name")
                          .required(true))

                    .arg( Arg::with_name("archive")
                          .help("Pick an archived project")
                          .short("a").long("archive")
                          .takes_value(true))

                    .arg( Arg::with_name("template")
                          .help("Edit a template (currently .tyml)")
                          .short("t").long("template"))

                    .arg( Arg::with_name("editor")
                          .help("Override the configured editor")
                          .short("e").long("editor")
                          .takes_value(true))
                   )

        .subcommand(SubCommand::with_name("show")
                    .about("Display a specific project")

                    .arg( Arg::with_name("search_term")
                          .help("Search term, possibly event name")
                          .required(true))

                    .arg( Arg::with_name("archive")
                          .help("Pick an archived project")
                          .short("a").long("archive")
                          .takes_value(true))

                    .arg( Arg::with_name("template")
                          .help("Show show fields in templates that are filled")
                          .short("t").long("template")
                          .conflicts_with("archive")
                          )
                   )

        .subcommand(SubCommand::with_name("new")
                    .arg( Arg::with_name("name")
                          .help("Project name")
                          .required(true))

                    .arg( Arg::with_name("template")
                          .help("Use a specific template")
                          .short("t").long("template")
                          .takes_value(true))

                    .arg( Arg::with_name("editor")
                          .help("Override the configured editor")
                          .short("e").long("editor")
                          .takes_value(true))

                    .arg( Arg::with_name("don't edit")
                          .help("Do not edit the file after creation")
                          .short("d"))

                    )

        //.subcommand(SubCommand::with_name("archive"))
        //.subcommand(SubCommand::with_name("unarchive"))

        .subcommand(SubCommand::with_name("config")
                    .about("Show and edit your config")

                    .arg( Arg::with_name("edit")
                          .help("Edit your config")
                          .short("e").long("edit")
                          )

                    .arg( Arg::with_name("show")
                          .help("Show a specific config value")
                          .short("s").long("show")
                          .takes_value(true))

                    .arg( Arg::with_name("default")
                          .help("Show default config")
                          .short("d").long("default")
                          )
                   )

        .subcommand(SubCommand::with_name("whoami"))

        .get_matches();

    // command: "new"
    if let Some(matches) = matches.subcommand_matches("new") {
        let name     = matches.value_of("name").unwrap();
        let editor   = CONFIG.get_path("editor").unwrap().as_str().unwrap();

        let template = matches.value_of("template").or(
            CONFIG.get_path("template").unwrap().as_str()
            ).unwrap();

        new_project(&name, &template, &editor, !matches.is_present("don't edit"));
    }

    // command: "list"
    else if let Some(matches) = matches.subcommand_matches("list") {
        if matches.is_present("templates"){
            list_templates(); }
        else {

        let mut sort = matches.value_of("sort").unwrap_or("index");

        // list archive of year `archive`
        let dir = if let Some(archive) = matches.value_of("archive"){
            let archive = archive.parse::<i32>().unwrap();
            LuigiDir::Archive(archive)
        }

        // or list all, but sort by date
        else if matches.is_present("all"){
            // sort by date on --all of not overriden
            if !matches.is_present("sort"){ sort = "date" }
            LuigiDir::All }

        // or list normal
        else { LuigiDir::Working };

        list_projects(dir, sort);
        }
    }

    // command: "edit"
    else if let Some(matches) = matches.subcommand_matches("edit") {
        let search_term = matches.value_of("search_term").unwrap();

        let editor = matches.value_of("editor").unwrap_or( CONFIG.get_path("editor").unwrap().as_str().unwrap());

        if matches.is_present("template"){
            edit_template(search_term,&editor);
        } else {
            if let Some(archive) = matches.value_of("archive"){
                let archive = archive.parse::<i32>().unwrap();
                edit_project(LuigiDir::Archive(archive), &search_term, &editor);
            } else {
                edit_project(LuigiDir::Working, &search_term, &editor);
            }
        }
    }

    // command: "show"
    else if let Some(matches) = matches.subcommand_matches("show") {
        let search_term = matches.value_of("search_term").unwrap();
        if let Some(archive) = matches.value_of("archive"){
            let archive = archive.parse::<i32>().unwrap();
            show_project(LuigiDir::Archive(archive), &search_term);
        } else if  matches.is_present("template"){
            show_template(search_term);
        } else {
            show_project(LuigiDir::Working, &search_term);
        }
    }

    // command: "config"
    else if let Some(matches) = matches.subcommand_matches("config") {
        if let Some(path) = matches.value_of("show"){
            config_show(&path);
        }
        else if matches.is_present("edit"){
            let editor = CONFIG.get_path("editor").unwrap().as_str().unwrap();
            config_edit(&editor); }
        else if matches.is_present("default"){ config_show_default(); }
    }

    // command: "whoami"
    else if  matches.is_present("whoami") {
        config_show("manager_name");
    }
}

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

    let project_paths = execute(||luigi.list_project_files(LuigiDir::Working));
    let projects: Vec<Project> = project_paths
        .iter()
        .map(|path| Project::open(path).unwrap())
        .collect();

    println!("{:#?}", repo.status);
    print::print_projects(print::status_rows(&projects,&repo));
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

fn sort_by_(option:&str, projects:&mut [Project]){
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

/// Opens up all projects to look inside and check content.
///
/// TODO This could be parallelized
/// TODO move this into `Luigi`
pub fn search_projects(dir:LuigiDir, search_term:&str) -> Vec<Project> {
    let luigi = setup_luigi();

    let projects: Vec<Project> = execute(||luigi.list_project_files(dir))
        .iter()
        .map(|path| Project::open(path).unwrap())
        .filter(|project| project.name().to_lowercase().contains(&search_term.to_lowercase()))
        .collect();

    projects
}

/// Command LIST [--archive, --all]
pub fn list_projects(dir:LuigiDir, sort:&str){
    let luigi = setup_luigi();
    let project_paths = execute(||luigi.list_project_files(dir));
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
    let projects: Vec<Project> = execute(||luigi.list_broken_projects(dir))
        .iter()
        .map(|p|Project::open(p).unwrap()).collect() ;
    print::print_projects(print::simple_rows(&projects));
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
    for project in search_projects(dir, &search_term){
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
