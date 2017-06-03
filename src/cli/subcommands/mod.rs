use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::{env, fs};
use std::io::Write;
use std::collections::HashMap;

use open;
use clap::ArgMatches;
use chrono::prelude::*;

use asciii;
use asciii::CONFIG;
use asciii::config;
use asciii::util;
use asciii::BillType;
use asciii::actions;
use asciii::storage::*;
use asciii::project::Project;
use asciii::actions::setup_storage;


// simple_rows, verbose_rows,
// path_rows, dynamic_rows,
// print_projects,print_csv};

pub mod git;
pub use self::git::*;

pub mod list;
pub use self::list::*;

pub mod show;
pub use self::show::*;

use super::{execute, fail};

#[cfg(feature="shell")] use super::shell;

// TODO refactor this into actions module and actual, short subcommands

/// Create NEW Project
// #[deprecated(note="move to asciii::actions")]
pub fn new(matches: &ArgMatches) {
    let project_name = matches.value_of("name").expect("You did not pass a \"Name\"!");
    let editor = CONFIG.get("user/editor").and_then(|e| e.as_str());

    let template_name = matches.value_of("template")
        .or(CONFIG.get("template").unwrap().as_str())
        .unwrap();

    let edit = !matches.is_present("don't edit");
    let luigi = execute(setup_storage);

    let mut fill_data: HashMap<&str, String> = HashMap::new();

    if let Some(description) = matches.value_of("description") {
        debug!("Filling in DESCRIPTION");
        fill_data.insert("DESCRIPTION", description.to_owned());
    }

    if let Some(date) = matches.value_of("date") {
        debug!("Filling in DATE-EVENT");
        fill_data.insert("DATE-EVENT", date.to_owned());
    }

    if let Some(time) = matches.value_of("time") {
        debug!("Filling in TIME-START");
        fill_data.insert("TIME-START", time.to_owned());
    }

    if let Some(time_end) = matches.value_of("time_end") {
        debug!("Filling in TIME-END");
        fill_data.insert("TIME-END", time_end.to_owned());
    }

    if let Some(manager) = matches.value_of("manager") {
        debug!("Filling in MANAGER");
        fill_data.insert("MANAGER", manager.to_owned());
    }

    let project = execute(|| luigi.create_project(project_name, template_name, &fill_data));
    let project_file = project.file();
    if edit {
        util::pass_to_command(&editor, &[project_file]);
    }
}

fn matches_to_dir<'a>(matches: &'a ArgMatches) -> StorageDir {
        if matches.is_present("archive"){
            let archive_year = matches.value_of("archive")
                                      .and_then(|y|y.parse::<i32>().ok())
                                      .unwrap_or(UTC::today().year());
            StorageDir::Archive(archive_year)
        }

        else if matches.is_present("year"){
            let year = matches.value_of("year")
                              .and_then(|y|y.parse::<i32>().ok())
                              .unwrap_or(UTC::today().year());
            StorageDir::Year(year)
        }

        // or list all, but sort by date
        else if matches.is_present("all"){
            // sort by date on --all of not overriden
            StorageDir::All }

        // or list normal
        else { StorageDir::Working }
}

fn matches_to_search<'a>(matches: &'a ArgMatches) -> (Vec<&'a str>, StorageDir) {
    let search_terms = matches
        .values_of("search_term")
        .map(|v| v.collect::<Vec<&str>>())
        .unwrap_or_else(Vec::new);

    debug!("matches_to_search: --archive={:?}", matches.value_of("archive"));


    let dir = matches_to_dir(matches);

    (search_terms, dir)
}

/// Produces a list of paths.
/// This is more general than `with_projects`, as this includes templates too.
pub fn matches_to_paths(matches: &ArgMatches, luigi: &Storage<Project>) -> Vec<PathBuf> {
    let search_terms = matches.values_of("search_term")
                              .map(|v| v.collect::<Vec<&str>>())
                              .unwrap_or_else(Vec::new);

    if matches.is_present("template") {
        super::execute(|| luigi.list_template_files())
            .into_iter()
            .filter(|f| {
                let stem = f.file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap_or("");
                search_terms.contains(&stem)
            })
            .collect::<Vec<_>>()
    } else {
        let dir = if let Some(archive) = matches.value_of("archive") {
            StorageDir::Archive(archive.parse::<i32>().unwrap())
        } else {
            StorageDir::Working
        };

        super::execute(|| luigi.search_projects_any(dir, &search_terms))
            .iter()
            .map(|project| project.dir())
            .collect::<Vec<_>>()

    }
}



/// Command CSV
pub fn csv(matches: &ArgMatches) {
    use chrono::{Local, Datelike};
    let year = matches.value_of("year")
                      .and_then(|y| y.parse::<i32>().ok())
                      .unwrap_or(Local::now().year());

    debug!("asciii csv --year {}", year);
    let csv = execute(|| actions::csv(year));
    println!("{}", csv);
}


/// Command EDIT
pub fn edit(matches: &ArgMatches) {
    let search_term = matches.value_of("search_term").unwrap();
    let search_terms = matches.values_of("search_term").unwrap().collect::<Vec<&str>>();

    let editor = matches.value_of("editor")
        .or(CONFIG.get("user/editor")
                  .and_then(|e| e.as_str()));

    if matches.is_present("template") {
        with_templates(search_term, |template_paths:&[PathBuf]| util::pass_to_command(&editor, template_paths));

    } else if let Some(archive) = matches.value_of("archive") {
        let archive = archive.parse::<i32>().unwrap();
        edit_projects(StorageDir::Archive(archive), &search_terms, &editor);
    } else {
        edit_projects(StorageDir::Working, &search_terms, &editor);
    }
}

fn edit_projects(dir: StorageDir, search_terms: &[&str], editor: &Option<&str>) {
    let luigi = execute(setup_storage);
    let mut all_projects = Vec::new();
    for search_term in search_terms {
        let mut paths = execute(|| luigi.search_projects(dir, search_term));
        if paths.is_empty() {
            // println!{"Nothing found for {:?}", search_term}
        } else {
            all_projects.append(&mut paths);
        }
    }

    if all_projects.is_empty() {
        fail(format!("Nothing found for {:?}", search_terms));
    } else {
        let all_paths = all_projects.iter().map(|p| p.file()).collect::<Vec<PathBuf>>();
        util::pass_to_command(&editor, &all_paths);
    }
}

/// Command WORKSPACE 
pub fn workspace(matches: &ArgMatches) {
    println!("{:?}", matches);
    let luigi = execute(setup_storage);
    let editor = matches.value_of("editor")
        .or(CONFIG.get("user/editor")
                  .and_then(|e| e.as_str()));
    util::pass_to_command(&editor, &[luigi.working_dir()]);
}

/// Command EDIT --template
pub fn with_templates<F>(name: &str, action: F)
    where F: FnOnce(&[PathBuf])
{
    let luigi = execute(setup_storage);
    let template_paths = execute(||luigi.list_template_files())
        .into_iter() // drain?
        .filter(|f|f.file_stem() .unwrap_or_else(||OsStr::new("")) == name)
        .collect::<Vec<PathBuf>>();
    action(template_paths.as_slice());
}

/// Command SET
pub fn set(m: &ArgMatches) {
    let field = m.value_of("field name")
                            .unwrap()
                            .chars()
                            .flat_map(|c| c.to_uppercase())
                            .collect::<String>();
    let value = m.value_of("field value").unwrap();
    let (search_terms, dir) = matches_to_search(m);

    execute(|| {
        actions::with_projects(dir, &search_terms, |project| {
            println!("{}: {}", project.short_desc(), project.empty_fields().join(", "));
            if !project.empty_fields().contains(&field) {
                return Err(format!("{:?} was not found in {}", field, project.short_desc()).into());
            }
            if util::really(&format!("do you want to set the field {} in {:?} [y|N]",
                                     field,
                                     project.short_desc())) {
                project.replace_field(&field, &value).map_err(|e| e.into())
            } else {
                Err("Don't want to".into())
            }
        })
    })
}


#[cfg(feature="document_export")]
fn infer_bill_type(m: &ArgMatches) -> Option<BillType> {
    match (m.is_present("offer"), m.is_present("invoice")) {
        (true, true)   => unreachable!("this should have been prevented by clap-rs"),
        (true, false)  => Some(BillType::Offer),
        (false, true)  => Some(BillType::Invoice),
        (false, false) => None,
    }
}

/// Command CALENDAR
pub fn calendar(matches: &ArgMatches) {
    let calendar = execute(||actions::calendar(matches_to_dir(matches), matches.is_present("tasks")));
    println!("{}", calendar);
}



/// Command SPEC
/// TODO make this not panic :D
/// TODO move this to `spec::all_the_things`
pub fn spec(_: &ArgMatches) {
    execute(actions::spec)
}

/// Command MAKE
#[cfg(feature="document_export")]
pub fn make(m: &ArgMatches) {
    let template_name = m.value_of("template").unwrap_or("document");
    let bill_type = infer_bill_type(m);
    let (search_terms, dir) = matches_to_search(m);

    debug!("make {t}({s}/{d:?}, invoice={i:?})",
           d = dir,
           s = search_terms[0],
           t = template_name,
           i = bill_type);

    execute(|| {
        actions::projects_to_doc(dir,
                                 search_terms[0],
                                 template_name,
                                 &bill_type,
                                 m.is_present("dry-run"),
                                 m.is_present("force"))
    });
}



/// Command DELETE
pub fn delete(m: &ArgMatches) {
    let (search_terms, dir) = matches_to_search(m);
    if m.is_present("template") {
        unimplemented!();
    } else {
        execute(|| actions::delete_project_confirmation(dir, &search_terms));
    }
}

#[cfg(not(feature="document_export"))]
pub fn make(_: &ArgMatches) {
    error!("Make functionality not built-in with this release!");
}







/// TODO make this be have like `edit`, taking multiple names
pub fn archive(matches: &ArgMatches) {
    if let Some(search_terms) = matches.values_of("search terms"){
        let search_terms = search_terms.collect::<Vec<_>>();
        let year = matches.value_of("year").and_then(|s| s.parse::<i32>().ok());
        let moved_files = execute(|| actions::archive_projects(&search_terms, year, matches.is_present("force")));
        debug!("archive({:?},{:?}) :\n{:?}", search_terms, year, moved_files);
    } else if matches.is_present("all"){
        debug!("archiving all I can find");
        let moved_files = execute(|| actions::archive_all_projects());
        debug!("git adding {:?} ", moved_files);
    } else {
        debug!("what do you wanna do?");
    }
}

pub fn unarchive(matches: &ArgMatches) {
    let year = matches.value_of("year").unwrap();
    let year = year.parse::<i32>()
        .unwrap_or_else(|e| panic!("can't parse year {:?}, {:?}", year, e));
    let search_terms = matches.values_of("name").unwrap().collect::<Vec<_>>();
    let moved_files = execute(|| actions::unarchive_projects(year, &search_terms));
    debug!("unarchive({:?},{:?}) :\n{:?}", search_terms, year, moved_files);
}

pub fn config(matches: &ArgMatches) {
    if let Some(path) = matches.value_of("show") {
        config_show(path);
    }

    if matches.is_present("location") {
        println!("config location: {:?}", config::ConfigReader::path_home())
    }

    else if matches.is_present("init") {
        let local = config::ConfigReader::path_home();
        println!("config location: {:?}", local);

        if local.exists() {
            error!("{:?} already exists, can't overwrite", local);
        } else {
            if let Ok(mut file) = fs::File::create(local){
                for line in config::DEFAULT_CONFIG.lines()
                    .take_while(|l| !l.contains("BREAK"))
                    {
                        file.write_fmt(format_args!("{}\n", line))
                            .expect("cannot write this line to the config file");
                    }
                let editor = matches.value_of("editor")
                    .or( CONFIG.get("user/editor").and_then(|e|e.as_str()));
                config_edit(&editor);
            }
        }

    }

    else if matches.is_present("edit") {
        let editor = matches.value_of("editor")
                            .or(CONFIG.get("user/editor")
                                      .and_then(|e| e.as_str()));
        config_edit(&editor);
    }

    else if matches.is_present("default") {
        config_show_default();
    }
}



/// Command CONFIG --show
pub fn config_show(path: &str) {
    println!("{}: {:#?}", path,
             CONFIG.get_to_string(&path)
                   .unwrap_or_else(|| format!("{} not set", path)));
}

/// Command CONFIG --edit
fn config_edit(editor: &Option<&str>) {
    let local = config::ConfigReader::path_home();
    if local.exists() {
        util::pass_to_command(editor, &[&CONFIG.path]);
    } else {
        error!("Cannot open {:?}, run `asciii config --init` to create it.", local)
    }
}

/// Command CONFIG --default
fn config_show_default() {
    println!("{}", config::DEFAULT_CONFIG);
}


/// Command DOC
pub fn doc() {
    open::that(asciii::DOCUMENTATION_URL).unwrap();
}

/// Command VERSION
pub fn version() {
    println!("{}", *asciii::VERSION);
}

/// Command DUES
pub fn dues(matches: &ArgMatches) {
    let dues = if matches.is_present("wages") {
        actions::open_wages()
    } else {
        actions::open_payments()
    };
    if let Ok(dues) = dues {
        println!("{}", dues.postfix());
    }
}

// pub fn open_path(matches:&ArgMatches){path(matches, |path| {open::that(path).unwrap();})}
pub fn open_path(m: &ArgMatches) {
    if m.is_present("search_term") {
        // let bill_type = infer_bill_type(m);
        // let template_name = "document";
        // let (search_terms, dir) = matches_to_search(m);
        unimplemented!()
    } else {
        path(m, |path| {
            open::that(path).unwrap();
        })
    }
}

pub fn path<F: Fn(&Path)>(m: &ArgMatches, action: F) {

    let path = CONFIG.get_str("path")
        .expect("Faulty config: field output_path does not contain a string value");
    let storage_path = CONFIG.get_str("dirs/storage")
        .expect("Faulty config: field output_path does not contain a string value");
    let templates_path = CONFIG.get_str("dirs/templates")
        .expect("Faulty config: field output_path does not contain a string value");
    let output_path = CONFIG.get_str("output_path")
        .expect("Faulty config: field output_path does not contain a string value");

    let exe = env::current_exe().unwrap();

    if m.is_present("templates") {
        action(&PathBuf::from(path)
            .join(storage_path)
            .join(templates_path)
            );
    }

    else if m.is_present("output") {
        action(&util::replace_home_tilde(Path::new(output_path)));
    }

    else if m.is_present("bin") {
        action(exe.parent().unwrap());
    }

    else {
        // default case
        let path = util::replace_home_tilde(Path::new(path)).join(storage_path);
        action(&path);
    }
}

#[cfg(feature="shell")]
pub fn shell(_matches: &ArgMatches) {
    shell::launch_shell();
}

#[cfg(not(feature="shell"))]
pub fn shell(_matches: &ArgMatches) {
    error!("Shell functionality not built-in with this release!");
}

