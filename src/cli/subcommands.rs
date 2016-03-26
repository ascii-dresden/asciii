use std::path::{Path,PathBuf};
use std::ffi::OsStr;
use std::env;
use std::process;

use clap::ArgMatches;
use config;
use super::super::CONFIG;
use terminal_size::{Width, terminal_size }; // TODO replace with other lib
use manager::{LuigiDir, LuigiProject,};
use project::Project;
use util;
use util::exit;
use super::print;

// TODO keep this up to date or find a way to make this dynamic
const STATUS_ROWS_WIDTH:u16 = 96;

/// Create NEW Project
pub fn new(matches:&ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("new") {
        let name     = matches.value_of("name").unwrap();
        let editor   = CONFIG.get_path("editor").unwrap().as_str().unwrap();

        let template = matches.value_of("template")
            .or( CONFIG.get_path("template").unwrap().as_str())
            .unwrap();

        new_project(&name, &template, &editor, !matches.is_present("don't edit"));
    }
}

fn new_project(project_name:&str, template_name:&str, editor:&str, edit:bool){
    let luigi = super::setup_luigi();
    //let project = execute(|| luigi.create_project::<Project>(&project_name, &template_name));
    let project = luigi.create_project::<Project>(&project_name, &template_name).unwrap();
    let project_file = project.file();
    if edit {
        util::open_in_editor(&editor, &[project_file]);
    }
}

/// Command LIST
pub fn list(matches:&ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("list") {
        if matches.is_present("templates"){
            list_templates();
        } else if matches.is_present("years"){
            list_years();
        } else {

            let mut sort = matches.value_of("sort").unwrap_or("index");

            // list archive of year `archive`
            let dir = if let Some(archive_year) = matches.value_of("archive"){
                let archive = archive_year.parse::<i32>().unwrap();
                LuigiDir::Archive(archive)
            }

            // or list all, but sort by date
            else if matches.is_present("all"){
                // sort by date on --all of not overriden
                if !matches.is_present("sort"){ sort = "date" }
                LuigiDir::All }

            // or list normal
            else { LuigiDir::Working };

            if matches.is_present("paths"){
                list_paths(dir);
            }
            else if matches.is_present("broken"){
                list_broken_projects(dir);
            }
            else {
                list_projects(dir, sort, matches.is_present("simple"));
            }
        }
    }
}


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
fn list_projects(dir:LuigiDir, sort:&str, simple:bool){
    let luigi = super::setup_luigi_with_git();
    let project_paths = super::execute(||luigi.list_project_files(dir));
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
fn list_broken_projects(dir:LuigiDir){
    use util::yaml::YamlError;
    let luigi = super::setup_luigi();
    let invalid_files = super::execute(||luigi.list_project_files(dir));
    let tups = invalid_files
        .iter()
        .filter_map(|dir| Project::open(dir).err().map(|e|(e, dir)) )
        .collect::<Vec<(YamlError,&PathBuf)>>();

    for (err,path) in tups{
        println!("{}: {}", path.display(), err);
    }
}

/// Command LIST --templates
fn list_templates(){
    let luigi = super::setup_luigi();
    let template_paths = super::execute(||luigi.list_template_files());

    for path in template_paths{
        if let Some(stem) = path.file_stem(){
            println!("{}", stem.to_string_lossy());
        } else {
            println!("broken template: {}", path.display());
        }
    }
}

/// Command LIST --years
fn list_years(){
    let luigi = super::setup_luigi();
    let years = super::execute(||luigi.list_years());
    println!("{:?}", years);
}




/// Command LIST --years
fn list_paths(dir:LuigiDir){
    let luigi = super::setup_luigi();
    let paths = super::execute(||luigi.list_project_files(dir));
    for path in paths{
        println!("{}", path.display());
    }
}

/// Command EDIT
pub fn edit(matches:&ArgMatches) {
    if let Some(matches) = matches.subcommand_matches("edit") {

        let search_term = matches.value_of("search_term").unwrap();
        let search_terms = matches.values_of("search_term").unwrap().collect::<Vec<&str>>();

        let editor = matches.value_of("editor").unwrap_or( CONFIG.get_path("editor").unwrap().as_str().unwrap());

        if matches.is_present("template"){
            edit_template(search_term,&editor);

        } else {
            if let Some(archive) = matches.value_of("archive"){
                let archive = archive.parse::<i32>().unwrap();
                edit_projects(LuigiDir::Archive(archive), &search_terms, &editor);
            } else {
                edit_projects(LuigiDir::Working, &search_terms, &editor);
            }
        }
    }
}

fn edit_projects(dir:LuigiDir, search_terms:&[&str], editor:&str){
    let luigi = super::setup_luigi();
    let mut all_paths = Vec::new();
    for search_term in search_terms{
        let mut paths = super::execute(||luigi.search_projects(dir.clone(), &search_term));
        if paths.is_empty(){
            //println!{"Nothing found for {:?}", search_term}
        } else {
            all_paths.append(&mut paths);
        }
    }

    if all_paths.is_empty(){
        println!{"Nothing found for {:?}", search_terms}
        process::exit(1);
    } else {
        util::open_in_editor(&editor, all_paths.as_slice());
    }
}

/// Command EDIT --template
fn edit_template(name:&str, editor:&str){
    let luigi = super::setup_luigi();
    let template_paths = super::execute(||luigi.list_template_files())
        .iter()
        .filter(|f|f.file_stem() .unwrap_or(&OsStr::new("")) == name)
        .cloned()
        .collect::<Vec<PathBuf>>();
    util::open_in_editor(&editor, &template_paths);
}


/// Command SHOW
pub fn show(matches:&ArgMatches){
    if let Some(matches) = matches.subcommand_matches("show") {
        let search_term = matches.value_of("search_term").unwrap();
        if let Some(year) = matches.value_of("archive"){
            let year = year.parse::<i32>().unwrap();
            show_project(LuigiDir::Archive(year), &search_term);
        } else if  matches.is_present("template"){
            show_template(search_term);
        } else {
            show_project(LuigiDir::Working, &search_term);
        }
    }
}

fn show_project(dir:LuigiDir, search_term:&str){
    let luigi = super::setup_luigi();
    for project in super::execute(||luigi.search_projects(dir, &search_term))
        .iter()
            .filter_map(|path| Project::open(path).ok())
            {
                print::show_items(&project);
            }
}




/// Command SHOW --template
use templater::Templater;
fn show_template(name:&str){
    let luigi = super::setup_luigi();
    let templater = Templater::new(&luigi.get_template_file(name).unwrap()).unwrap();
    println!("{:#?}", templater.list_keywords());
}
pub fn archive(matches:&ArgMatches){
    if let Some(matches) = matches.subcommand_matches("archive") {
        let name = matches.value_of("NAME").unwrap();
        let year = matches.value_of("year")
            .and_then(|s|s.parse::<i32>().ok());
        archive_project(&name, year);
    }
}

pub fn unarchive(matches:&ArgMatches){
    if let Some(matches) = matches.subcommand_matches("unarchive") {
        let year = matches.value_of("YEAR").unwrap();
        let name = matches.value_of("NAME").unwrap();
        let year = year.parse::<i32>().unwrap_or_else(|e|panic!("can't parse year {:?}, {:?}", year, e));
        unarchive_project(year, &name);
    }
}

pub fn config(matches:&ArgMatches){
    if let Some(matches) = matches.subcommand_matches("config") {
        if let Some(path) = matches.value_of("show"){
            config_show(&path);
        }

        else if matches.is_present("edit"){
            let editor = CONFIG.get_path("editor").unwrap().as_str().unwrap();
            config_edit(&editor); }

        else if matches.is_present("default"){ config_show_default(); }
    }

    // command: "whoami"
    if matches.is_present("whoami") {
        config_show("manager_name");
    }
}

/// Command ARCHIVE <NAME>
fn archive_project(name:&str, manual_year:Option<i32>){
    let luigi = super::setup_luigi();
    if let Ok(projects) = luigi.search_projects(LuigiDir::Working, name){
        for project in projects.iter().filter_map(|path| Project::open(path).ok()) {
            if project.valid_stage3().is_ok(){
                let year = manual_year.or(project.year()).unwrap();
                println!("archiving {} ({})",  project.ident(), project.year().unwrap());
                super::execute(||luigi.archive_project(&project, year));
            }
            else {
                println!("CANNOT archive {} ({})",
                project.ident(), project.file().display());
            }
        }
    }
}


/// Command UNARCHIVW <YEAR> <NAME>
fn unarchive_project(year:i32, name:&str){
    let luigi = super::setup_luigi();
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
fn config_show(path:&str){
    //TODO config_show could be prettier
    println!("{:#?}", CONFIG.get_str(&path));
}

/// Command CONFIG --edit
fn config_edit(editor:&str){
    //println!("{:?}{:?}", &editor, vec![CONFIG.path.to_str().unwrap().to_owned()]);
    util::open_in_editor(&editor, &[CONFIG.path.to_owned()]);
}

/// Command CONFIG --default
fn config_show_default(){
    println!("{}", config::DEFAULT_CONFIG);
}

pub fn path(matches:&ArgMatches){
    if let Some(matches) = matches.subcommand_matches("path") {
        if matches.is_present("templates"){
            println!("{}", PathBuf::from(CONFIG.get_str("path"))
                     .join( CONFIG.get_str("dirs/storage"))
                     .join( CONFIG.get_str("dirs/templates"))
                     .display());
        }
        else if matches.is_present("output"){
            println!("{}", CONFIG.get_str("output_path"));
        }
        else if matches.is_present("bin"){
            println!("{}", env::current_exe().unwrap().display());
        }
        else { // default case
            let path = util::replace_home_tilde(Path::new(CONFIG.get_str("path")))
                .join( CONFIG.get_str("dirs/storage"));
            println!("{}", path.display());
        }
    }
}



pub fn git(matches:&ArgMatches){
    if matches.is_present("status") {
        git_status();
    }

    if let Some(matches) = matches.subcommand_matches("add") {
        let search_terms = matches
            .values_of("search_term")
            .unwrap()
            .collect::<Vec<&str>>();
        git_add(&search_terms);
    }

    if matches.is_present("commit") {
        git_commit();
    }

    if matches.is_present("pull") {
        git_pull();
    }

    if matches.is_present("remote") {
        git_remote();
    }
}
/// Command STATUS
fn git_status(){
    let luigi = super::setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    //exit(repo.status()) // FIXME this does not behave right
}

/// Command COMMIT
fn git_commit(){
    let luigi = super::setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    exit(repo.commit())
}

/// Command REMOTE
/// exact replica of `git remote -v`
fn git_remote(){
    let luigi = super::setup_luigi_with_git();
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

/// Command ADD
fn git_add(search_terms:&[&str]){
    let luigi = super::setup_luigi_with_git();
    let projects = luigi.search_multiple_projects(LuigiDir::Working, search_terms)
        .unwrap()
        .iter()
        .filter_map(|path| match Project::open(path){
            Ok(project) => Some(project),
            Err(err) => {
                println!("Erroneous Project: {}\n {}", path.display(), err);
                None
            }
        })
        .map(|project|project.dir())
        .collect::<Vec<PathBuf>>();
    let repo = luigi.repository.unwrap();
    exit(repo.add(&projects));
}

/// Command PULL
fn git_pull(){
    let luigi = super::setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    exit(repo.pull())
}
