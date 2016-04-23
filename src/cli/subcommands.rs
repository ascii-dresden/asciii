use std::path::{Path,PathBuf};
use std::ffi::OsStr;
use std::env;
use std::collections::HashMap;

use clap::ArgMatches;
use open;

use config;
use ::CONFIG;
use terminal_size::{Width, terminal_size }; // TODO replace with other lib
use storage::*;
use project::Project;
use util;
use super::{print,setup_luigi, setup_luigi_with_git, fail};
use super::{ListConfig, ListMode};

// TODO keep this up to date or find a way to make this dynamic
const STATUS_ROWS_WIDTH:u16 = 96;

/// Create NEW Project
pub fn new(matches:&ArgMatches){
    #![allow(unreachable_code,unused_variables)]
    let project_name     = matches.value_of("name").expect("You did not pass a \"Name\"!");
    let editor   = CONFIG.get_path("editor").and_then(|e|e.as_str());

    let template_name = matches.value_of("template")
        .or( CONFIG.get_path("template").unwrap().as_str())
        .unwrap();

    let edit = !matches.is_present("don't edit");
    let luigi = setup_luigi();

    let mut fill_data:HashMap<&str, String> = HashMap::new();

    if let Some(description) = matches.value_of("description"){
        fill_data.insert("DESCRIPTION", description.to_owned());
    }

    if let Some(date) = matches.value_of("date"){
        fill_data.insert("DATE-EVENT", date.to_owned());
    }

    if let Some(time) = matches.value_of("time"){
        fill_data.insert("TIME-START", time.to_owned());
    }

    if let Some(time_end) = matches.value_of("time_end"){
        fill_data.insert("TIME-END", time_end.to_owned());
    }

    if let Some(manager) = matches.value_of("manager"){
        fill_data.insert("MANAGER", manager.to_owned());
    }

    let project = super::execute(|| luigi.create_project(&project_name, &template_name, &fill_data));
    let project_file = project.file();
    if edit {
        util::open_in_editor(&editor, &[project_file]);
    }
}

fn decide_mode(simple:bool, verbose:bool, paths:bool,nothing:bool, csv:bool) -> ListMode{
    if csv{     ListMode::Csv } else
    if nothing{ ListMode::Nothing } else
    if paths{   ListMode::Paths } else
    {
        match (simple, verbose, CONFIG.get_bool("list/verbose")){
            (false, true,  _   ) => {debug!("-v overwrites config"); ListMode::Verbose },
            (false,    _, true ) => {debug!("-v from config");ListMode::Verbose},
                          _      => {debug!("simple mode");ListMode::Simple},
        }
    }
}


/// Command LIST
pub fn list(matches:&ArgMatches){
    if matches.is_present("templates"){
        list_templates();
    } else if matches.is_present("years"){
        list_years();
    } else if matches.is_present("virtual_fields"){
        list_virtual_fields();
    }

    else {
        let list_mode = decide_mode(
            matches.is_present("simple"),
            matches.is_present("verbose"),
            matches.is_present("paths"),
            matches.is_present("nothing"),
            matches.is_present("csv")
            );

        let mut list_config = ListConfig{
            sort_by:   matches.value_of("sort") .unwrap_or_else(||CONFIG.get_str("list/sort")),
            mode:      list_mode,
            details:   matches.values_of("details").map(|v|v.collect::<Vec<&str>>()),
            filter_by: matches.values_of("filter").map(|v|v.collect::<Vec<&str>>()),

            ..Default::default()
        };
        //println!("list_config: {:#?}", list_config);

        // list archive of year `archive`
        let dir =
            if let Some(archive_year) = matches.value_of("archive"){
                let archive = archive_year.parse::<i32>().unwrap();
                StorageDir::Archive(archive)
            }

            // or list all, but sort by date
            else if matches.is_present("all"){
                // sort by date on --all of not overriden
                if !matches.is_present("sort"){ list_config.sort_by = "date" }
                StorageDir::All }

            // or list normal
            else { StorageDir::Working };

        if matches.is_present("broken"){
            list_broken_projects(dir);
        } else {
            list_projects(dir, &list_config);
        }
    }
}

/// Command LIST [--archive, --all]
///
/// This interprets the `ListConfig` struct and passes it on to either
///
/// * `print::rows()`
/// * `print::simple_rows()`
/// * `print::verbose_rows()`
///
/// which it prints with `print::print_projects()`
fn list_projects(dir:StorageDir, list_config:&ListConfig){
    use super::print::{simple_rows, verbose_rows,
                        path_rows, dynamic_rows,
                       print_projects,print_csv};

    let luigi = if CONFIG.get_bool("list/gitstatus"){
        setup_luigi_with_git()
    } else {
        setup_luigi()
    };

    let mut projects = super::execute(||luigi.open_project_files(dir));

    // filtering, can you read this
    if let Some(ref filters) = list_config.filter_by{
        projects.filter_by_all(filters);
    }

    // sorting
    super::sort_by(&mut projects,list_config.sort_by);

    // fit screen
    let wide_enough = match terminal_size() {
        Some((Width(w), _)) if w >= STATUS_ROWS_WIDTH => true,
        _ => false
    };

    if !wide_enough && list_config.mode != ListMode::Csv { // TODO room for improvement
        print_projects(simple_rows(&projects, &list_config));
    } else {
        debug!("list_mode: {:?}", list_config.mode );
        match list_config.mode{
            ListMode::Csv     => print_csv(&projects),
            ListMode::Paths   => print_projects(path_rows(&projects, &list_config)),
            ListMode::Simple  => print_projects(simple_rows(&projects, &list_config)),
            ListMode::Verbose => print_projects(verbose_rows(&projects,&list_config,luigi.repository)),
            ListMode::Nothing => print_projects(dynamic_rows(&projects,&list_config,luigi.repository)),
        }
    }
}

/// Command LIST --broken
fn list_broken_projects(dir:StorageDir){
    let luigi = setup_luigi();
    let invalid_files = super::execute(||luigi.list_project_files(dir));
    let tups = invalid_files
        .iter()
        .filter_map(|dir| Project::open(dir).err().map(|e|(e, dir)) )
        .collect::<Vec<(StorageError,&PathBuf)>>();

    for (err,path) in tups{
        println!("{}: {:?}", path.display(), err);
    }
}

/// Command LIST --templates
fn list_templates(){
    let luigi = setup_luigi();

    for name in super::execute(||luigi.list_template_names()){
        println!("{}", name);
    }
}

/// Command LIST --years
fn list_years(){
    let luigi = setup_luigi();
    let years = super::execute(||luigi.list_years());
    println!("{:?}", years);
}

/// Command LIST --virt
fn list_virtual_fields(){
    use ::project::spec::VirtualField;
    println!("{:?}", VirtualField::iter_variant_names().filter(|v|*v!="Invalid").collect::<Vec<&str>>());
}




/// Command EDIT
pub fn edit(matches:&ArgMatches) {

        let search_term = matches.value_of("search_term").unwrap();
        let search_terms = matches.values_of("search_term").unwrap().collect::<Vec<&str>>();

        let editor = matches.value_of("editor")
            .or( CONFIG.get_path("editor").and_then(|e|e.as_str()));

        if matches.is_present("template"){
            edit_template(search_term, &editor);

        } else {
            if let Some(archive) = matches.value_of("archive"){
                let archive = archive.parse::<i32>().unwrap();
                edit_projects(StorageDir::Archive(archive), &search_terms, &editor);
            } else {
                edit_projects(StorageDir::Working, &search_terms, &editor);
            }
        }
}

fn edit_projects(dir:StorageDir, search_terms:&[&str], editor:&Option<&str>){
    let luigi = setup_luigi();
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
        fail(format!("Nothing found for {:?}", search_terms));
    } else {
        util::open_in_editor(&editor, all_paths.as_slice());
    }
}

/// Command EDIT --template
fn edit_template(name:&str, editor:&Option<&str>){
    let luigi = setup_luigi();
    let template_paths = super::execute(||luigi.list_template_files())
        .iter()
        .filter(|f|f.file_stem() .unwrap_or(&OsStr::new("")) == name)
        .cloned()
        .collect::<Vec<PathBuf>>();
    util::open_in_editor(&editor, &template_paths);
}


/// Command SHOW
pub fn show(matches:&ArgMatches){
        let search_term = matches.value_of("search_term").unwrap();
        if let Some(year) = matches.value_of("archive"){
            let year = year.parse::<i32>().unwrap();
            show_project(StorageDir::Archive(year), &search_term);
        } else if  matches.is_present("template"){
            show_template(search_term);
        } else {
            show_project(StorageDir::Working, &search_term);
        }
}

fn show_project(dir:StorageDir, search_term:&str){
    // TODO make this use ProjectList
    let luigi = setup_luigi();
    let projects = super::execute(||luigi.search_projects(dir, &search_term));
    if !projects.is_empty(){
        for project in projects.iter().filter_map(|path| Project::open(path).ok()){
            print::show_items(&project);
        }
    } else{
        fail(format!("Nothing found for {:?}", search_term));
    }
}




/// Command SHOW --template
use templater::Templater;
fn show_template(name:&str){
    let luigi = setup_luigi();
    let templater = Templater::from_file(&luigi.get_template_file(name).unwrap()).unwrap();
    println!("{:#?}", templater.list_keywords());
}

pub fn archive(matches:&ArgMatches){
        let name = matches.value_of("NAME").unwrap();
        let year = matches.value_of("year")
            .and_then(|s|s.parse::<i32>().ok());
        archive_project(&name, year, matches.is_present("force"));
}

pub fn unarchive(matches:&ArgMatches){
        let year = matches.value_of("YEAR").unwrap();
        let name = matches.value_of("NAME").unwrap();
        let year = year.parse::<i32>().unwrap_or_else(|e|panic!("can't parse year {:?}, {:?}", year, e));
        unarchive_project(year, &name);
}

pub fn config(matches:&ArgMatches){
        if let Some(path) = matches.value_of("show"){
            config_show(&path);
        }

        else if matches.is_present("edit"){
            let editor = matches.value_of("editor")
                .or( CONFIG.get_path("editor").and_then(|e|e.as_str()));
            config_edit(&editor); }

        else if matches.is_present("default"){ config_show_default(); }
}

/// Command ARCHIVE <NAME>
// TODO perhaps move this code out of `::cli`
fn archive_project(name:&str, manual_year:Option<i32>, force:bool){
    let luigi = setup_luigi();
    debug!("using the force: {}", force);
    if let Ok(projects) = luigi.search_projects(StorageDir::Working, name){
        if projects.is_empty(){ fail(format!("Nothing found for {:?}", name)); }
        for project in projects.iter().filter_map(|path| Project::open(path).ok()) {
            if project.valid_stage3().is_ok() || force{
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
    else{
        fail(format!("Nothing found for {:?}", name));
    }
}


/// Command UNARCHIVW <YEAR> <NAME>
fn unarchive_project(year:i32, name:&str){
    let luigi = setup_luigi();
    if let Ok(projects) = luigi.search_projects(StorageDir::Archive(year), name){
        if projects.len() == 1 {
            println!("{:?}", projects);
            luigi.unarchive_project_file(&projects[0]).unwrap();
        } else{
            fail(format!("Ambiguous: multiple matches: {:#?}", projects ));
        }
    }
}


/// Command CONFIG --show
pub fn config_show(path:&str){
    println!("{}: {:#?}", path, CONFIG.get_path(&path)
             .map(|v|v.to_string())
             .unwrap_or_else(||format!("{} not set", path)));
}

/// Command CONFIG --edit
fn config_edit(editor:&Option<&str>){
    util::open_in_editor(&editor, &[CONFIG.path.to_owned()]);
}

/// Command CONFIG --default
fn config_show_default(){
    println!("{}", config::DEFAULT_CONFIG);
}

/// Command DOC
pub fn doc(){
    open::that("http://hoodie.github.io/asciii-rs/").unwrap();
}

/// Command TERM
pub fn term(){
    use terminal_size::{Width, Height, terminal_size };
    if let Some((Width(w), Height(h))) = terminal_size() {
        println!("Your terminal is {} cols wide and {} lines tall", w, h);
    } else {
        println!("Unable to get terminal size");
    }
}

pub fn show_path(matches:&ArgMatches){path(matches, |path| println!("{}", path.display()))}
pub fn open_path(matches:&ArgMatches){path(matches, |path| {open::that(path).unwrap();})}

pub fn path<F:Fn(&Path)>(matches:&ArgMatches, action:F){
        if matches.is_present("templates"){
            action(&PathBuf::from(CONFIG.get_str("path"))
                     .join( CONFIG.get_str("dirs/storage"))
                     .join( CONFIG.get_str("dirs/templates")));
        }
        else if matches.is_present("output"){
            action(
                    &util::replace_home_tilde(
                Path::new(
                    CONFIG.get_str("output_path")
                    )
                    ));
        }
        else if matches.is_present("bin"){
            action(&env::current_exe().unwrap());
        }
        else { // default case
            let path = util::replace_home_tilde(Path::new(CONFIG.get_str("path")))
                .join( CONFIG.get_str("dirs/storage"));
            action(&path);
        }
}



/// Command STATUS
pub fn git_status(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    util::exit(repo.status()) // FIXME this does not behave right
}

/// Command COMMIT
pub fn git_commit(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    util::exit(repo.commit())
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

/// Command ADD
pub fn git_add(matches:&ArgMatches){
    let luigi = setup_luigi_with_git();
    let search_terms = matches
        .values_of("search_term")
        .unwrap()
        .collect::<Vec<&str>>();

    debug!("adding {:?}", search_terms);

    let projects = luigi.search_multiple_projects(StorageDir::Working, &search_terms)
        .unwrap()
        .iter()
        .filter_map(|path| match Project::open(path){ // TODO use ProjectList
            Ok(project) => Some(project),
            Err(err) => {
                println!("Erroneous Project: {}\n {:#?}", path.display(), err);
                None
            }
        })
        .map(|project|project.dir())
        .collect::<Vec<PathBuf>>();
    let repo = luigi.repository.unwrap();
    util::exit(repo.add(&projects));
}

/// Command PULL
pub fn git_pull(){
    // TODO this doesn't need _with_git
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    util::exit(repo.pull())
}

/// Command PUSH
pub fn git_push(){
    let luigi = setup_luigi_with_git();
    let repo = luigi.repository.unwrap();
    util::exit(repo.push())
}
