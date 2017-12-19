use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::{env, fs};
use std::io;
use std::io::Write;
use std::collections::HashMap;

use open;
use clap::ArgMatches;
use chrono::prelude::*;

use asciii::{self, CONFIG, config, util, actions};
use asciii::project::Project;
use asciii::storage::*;
use asciii::templater::Templater;

#[cfg(feature="document_export")] use asciii::document_export;
#[cfg(feature="document_export")] use asciii::BillType;

// simple_rows, verbose_rows,
// path_rows, dynamic_rows,
// print_projects,print_csv};

pub mod git;
pub use self::git::*;

pub mod list;
pub use self::list::*;

pub mod show;
pub use self::show::*;

use ::cli::error::*;

#[cfg(feature="shell")] use super::shell;

// TODO refactor this into actions module and actual, short subcommands

/// Create NEW Project
// #[deprecated(note="move to asciii::actions")]
pub fn new(matches: &ArgMatches) -> Result<()> {
    let project_name = matches.value_of("name").expect("You did not pass a \"Name\"!");
    let editor = CONFIG.get("user/editor").and_then(|e| e.as_str());

    let template_name = matches.value_of("template")
        .or(CONFIG.get("template").unwrap().as_str())
        .unwrap();

    let edit = !matches.is_present("don't edit");
    let storage = setup::<Project>()?;

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

    let project_file = storage.create_project(project_name, template_name, &fill_data)?.file();
    if edit {
        util::pass_to_command(editor, &[project_file]);
    }
    Ok(())
}

fn matches_to_selection<'a>(matches: &'a ArgMatches) -> StorageSelection {
    let (search_terms, dir) = matches_to_search(matches);
    StorageSelection::DirAndSearch(dir, search_terms.into_iter().map(ToOwned::to_owned).collect())
}

fn matches_to_dir<'a>(matches: &'a ArgMatches) -> StorageDir {
        if matches.is_present("archive"){
            let archive_year = matches.value_of("archive")
                                      .and_then(|y|y.parse::<i32>().ok())
                                      .unwrap_or(Utc::today().year());
            StorageDir::Archive(archive_year)
        }

        else if matches.is_present("year"){
            let year = matches.value_of("year")
                              .and_then(|y|y.parse::<i32>().ok())
                              .unwrap_or(Utc::today().year());
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
pub fn matches_to_paths(matches: &ArgMatches, storage: &Storage<Project>) -> Result<Vec<PathBuf>> {
    let search_terms = matches.values_of("search_term")
                              .map(|v| v.collect::<Vec<&str>>())
                              .unwrap_or_else(Vec::new);

    if matches.is_present("template") {
        Ok(storage.list_template_files()?
            .into_iter()
            .filter(|f| {
                let stem = f.file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap_or("");
                search_terms.contains(&stem)
            })
            .collect::<Vec<_>>())
    } else {
        let dir = if let Some(archive) = matches.value_of("archive") {
            StorageDir::Archive(archive.parse::<i32>().unwrap())
        } else {
            StorageDir::Working
        };

        Ok(storage.search_projects_any(dir, &search_terms)?
            .iter()
            .map(|project| project.dir())
            .collect::<Vec<_>>())
    }
}



/// Command BOOTSTRAP
pub fn bootstrap(matches: &ArgMatches) -> Result<()> {

    let repo = matches.value_of("repo").unwrap();
    let editor = matches.value_of("editor")
                        .or( CONFIG.get("user.editor")
                                   .and_then(|e|e.as_str()));

    let default_to = get_storage_path()
        .to_str()
        .map(ToString::to_string)
        .expect("storage page not accessible");

    let to = matches.value_of("to").unwrap_or(&default_to);
    trace!("cloning {:?} to {:?}", repo, to);
    actions::clone_remote(repo, to)?;
    config_init(editor);

    Ok(())
}


/// Command CSV
pub fn csv(matches: &ArgMatches) -> Result<()> {
    use chrono::{Local, Datelike};
    let year = matches.value_of("year")
                      .and_then(|y| y.parse::<i32>().ok())
                      .unwrap_or(Local::now().year());

    debug!("asciii csv --year {}", year);
    let csv = actions::csv(year)?;
    println!("{}", csv);
    Ok(())
}


/// Command EDIT
pub fn edit(matches: &ArgMatches) -> Result<()> {
    let search_term = matches.value_of("search_term").unwrap();
    let search_terms = matches.values_of("search_term").unwrap().collect::<Vec<&str>>();

    let editor = matches.value_of("editor")
        .or(CONFIG.get("user/editor")
                  .and_then(|e| e.as_str()));

    if matches.is_present("template") {
        with_templates(search_term,
                       |template_paths:&[PathBuf]| util::pass_to_command(editor, template_paths)
                       )?;

    } else if let Some(archive) = matches.value_of("archive") {
        let archive = archive.parse::<i32>().unwrap();
        edit_projects(StorageDir::Archive(archive), &search_terms, editor)?;
    } else {
        edit_projects(StorageDir::Working, &search_terms, editor)?;
    }
    Ok(())
}


fn edit_projects(dir: StorageDir, search_terms: &[&str], editor: Option<&str>) -> Result<()> {
    let storage = setup::<Project>()?;
    let mut all_projects = Vec::new();
    for search_term in search_terms {
        let mut paths = storage.search_projects(dir, search_term)?;
        if paths.is_empty() {
            // println!{"Nothing found for {:?}", search_term}
        } else {
            all_projects.append(&mut paths);
        }
    }

    if all_projects.is_empty() {
        bail!("Nothing found for {:?}", search_terms);
    } else {
        let all_paths = all_projects.iter().map(|p| p.file()).collect::<Vec<PathBuf>>();
        util::pass_to_command(editor, &all_paths);
        Ok(())
    }
}

/// Command META
pub fn meta(matches: &ArgMatches) -> Result<()> {
    let storage = setup::<Project>()?;
    if let Some(matches) = matches.subcommand_matches("edit") {
        let editor = matches.value_of("editor")
                            .or(CONFIG.get("user.editor")
                                      .and_then(|e|e.as_str()));
        trace!("--> editing");
        if let Ok(path) = storage.get_extra_file("meta.toml") {
            util::pass_to_command(editor, &[path]);
        }
    }

    if let Some(matches) = matches.subcommand_matches("store") {
        trace!("--> storing");
        actions::store_meta()?;
    }

    if let Some(matches) = matches.subcommand_matches("dump") {
        trace!("--> dumping");
        let meta = actions::parse_meta();
        println!("{:#?}", meta);
    }
    Ok(())
}

/// Command WORKSPACE
pub fn workspace(matches: &ArgMatches) -> Result<()> {
    println!("{:?}", matches);
    let storage = setup::<Project>()?;

    let editor = matches.value_of("editor")
        .or(CONFIG.get("user/editor")
                  .and_then(|e| e.as_str()));
    util::pass_to_command(editor, &[storage.working_dir()]);
    Ok(())
}

/// Command EDIT --template
pub fn with_templates<F>(name: &str, action: F) -> Result<()>
    where F: FnOnce(&[PathBuf])
{
    let template_paths = setup::<Project>()?.list_template_files()?
        .into_iter() // drain?
        .filter(|f|f.file_stem() .unwrap_or_else(||OsStr::new("")) == name)
        .collect::<Vec<PathBuf>>();
    action(template_paths.as_slice());
    Ok(())
}

/// Command SET
pub fn set(m: &ArgMatches) -> Result<()> {
    let field = m.value_of("field name")
                            .unwrap()
                            .chars()
                            .flat_map(|c| c.to_uppercase())
                            .collect::<String>();
    let value = m.value_of("field value").unwrap();
    let (search_terms, dir) = matches_to_search(m);

    Ok(actions::with_projects(dir, &search_terms, |project| {
        println!("{}: {}", project.short_desc(), project.empty_fields().join(", "));
        if !project.empty_fields().contains(&field) {
            return Err(format!("{:?} was not found in {}", field, project.short_desc()).into());
        }
        if util::really(&format!("do you want to set the field {} in {:?}",
                                 field,
                                 project.short_desc())) {
            project.replace_field(&field, &value).map_err(|e| e.into())
        } else {
            Err("Don't want to".into())
        }
    })?)
}


/// Command CALENDAR
pub fn calendar(matches: &ArgMatches) -> Result<()> {
    let calendar = actions::calendar_with_tasks(matches_to_dir(matches), matches.is_present("tasks"))?;
    println!("{}", calendar);
    Ok(())
}



/// Command SPEC
/// TODO make this not panic :D
/// TODO move this to `spec::all_the_things`
pub fn spec(_: &ArgMatches) -> Result<()> {
    Ok(actions::spec()?)
}


#[cfg(feature="document_export")]
use self::document_export::ExportConfig;

#[cfg(feature="document_export")]
fn infer_bill_type(m: &ArgMatches) -> Option<BillType> {
    match (m.is_present("offer"), m.is_present("invoice")) {
        (true, true)   => unreachable!("this should have been prevented by clap-rs"),
        (true, false)  => Some(BillType::Offer),
        (false, true)  => Some(BillType::Invoice),
        (false, false) => None,
    }
}

#[cfg(feature="document_export")]
fn matches_to_export_config<'a>(m: &'a ArgMatches) -> Option<ExportConfig<'a>> {

    let template_name = m.value_of("template")
                         .or_else(||CONFIG.get("document_export/default_template").and_then(|e| e.as_str()))
                         .unwrap();
    let bill_type = infer_bill_type(m);

    let mut config = ExportConfig {
            select:        StorageSelection::Unintiailzed,
            template_name: template_name,
            bill_type:     bill_type,
            output:        m.value_of("output").map(Path::new),
            dry_run:       m.is_present("dry-run"),
            force:         m.is_present("force"),
            print_only:    m.is_present("print-only"),
            open:          m.is_present("open")
        };

    if  m.is_present("search_term") {
        let (search_terms, dir) = matches_to_search(m);
        let search_terms = search_terms.into_iter().map(ToOwned::to_owned).collect::<Vec<_>>();
        debug!("make {t}({s}/{d:?}, invoice={i:?})", d = dir, s = search_terms[0], t = template_name, i = bill_type);
        config.select = StorageSelection::DirAndSearch(dir, search_terms);
        Some(config)

    } else if let Some(file_path) = m.value_of("file") {
        debug!("make {t}({d:?}, invoice={i:?})", d = file_path, t = template_name, i = bill_type);
        config.select = StorageSelection::Paths(vec![PathBuf::from(file_path)]);
        Some(config)

    } else {
        error!("{}", lformat!("You have to provide either a search term or path"));
        None
    }

}


/// Command MAKE
#[cfg(feature="document_export")]
pub fn make(m: &ArgMatches) -> Result<()> {
    debug!("{:?}", m);
    if let Some(ref config) = matches_to_export_config(m) {
        Ok(document_export::projects_to_doc(config)?) // TODO if-let this TODO should return Result
    } else {
        Ok(())
    }
}



/// Command DELETE
pub fn delete(m: &ArgMatches) -> Result<()> {
    let (search_terms, dir) = matches_to_search(m);
    if m.is_present("template") {
        unimplemented!();
    } else {
        Ok(actions::delete_project_confirmation(dir, &search_terms)?)
    }
}

#[cfg(not(feature="document_export"))]
pub fn make(_: &ArgMatches) -> Result<()> {
    error!("Make functionality not built-in with this release!");
    Ok(())
}







/// TODO make this be have like `edit`, taking multiple names
pub fn archive(matches: &ArgMatches) -> Result<()> {
    if let Some(search_terms) = matches.values_of("search terms"){
        let search_terms = search_terms.collect::<Vec<_>>();
        let year = matches.value_of("year").and_then(|s| s.parse::<i32>().ok());
        let moved_files = actions::archive_projects(&search_terms, year, matches.is_present("force"))?;
        debug!("archive({:?},{:?}) :\n{:?}", search_terms, year, moved_files);
    } else if matches.is_present("all"){
        debug!("archiving all I can find");
        let moved_files = actions::archive_all_projects()?;
        debug!("git adding {:?} ", moved_files);
    } else {
        debug!("what do you wanna do?");
    }
    Ok(())
}

pub fn unarchive(matches: &ArgMatches) -> Result<()> {
    let year = matches.value_of("year").unwrap();
    let year = year.parse::<i32>()
        .unwrap_or_else(|e| panic!("can't parse year {:?}, {:?}", year, e));
    let search_terms = matches.values_of("name").unwrap().collect::<Vec<_>>();
    let moved_files = actions::unarchive_projects(year, &search_terms)?;
    debug!("unarchive({:?},{:?}) :\n{:?}", search_terms, year, moved_files);
    Ok(())
}

pub fn config(matches: &ArgMatches) -> Result<()> {
    let editor = matches.value_of("editor")
                        .or(CONFIG.get("user.editor")
                                  .and_then(|e|e.as_str()));

    if let Some(path) = matches.value_of("show") {
        config_show(path)?;
    }

    if matches.is_present("location") {
        println!("config location: {:?}", config::ConfigReader::path_home())
    }

    else if matches.is_present("init") {
        config_init(editor);
    }

    else if matches.is_present("edit") {
        config_edit(editor);
    }

    else if matches.is_present("default") {
        config_show_default()?;
    }
    Ok(())
}


/// Command CONFIG --init
///
/// # Warning! Interactive
/// This command will prompt the user for input on the commandline
///
pub fn config_init(editor: Option<&str>) {
    let local = config::ConfigReader::path_home();

    if local.exists() {
        error!("{:?} already exists, can't overwrite", local);

    } else if let Ok(mut file) = fs::File::create(local){

        let content;
        let mut template = Templater::new(config::DEFAULT_CONFIG).finalize();
        trace!("default config keywords: {:#?}", template.list_keywords());

        if util::really(&lformat!("do you want to set your name?")) {
            let name = util::git_user_name().and_then(|user_name| {
                if util::really(&lformat!("Is your name {:?}", user_name)) {
                    Some(user_name)
                } else {
                    None
                }
            }).unwrap_or_else(||{
                println!("{}", lformat!("What is your name?"));
                let mut your_name = String::new();
                io::stdin().read_line(&mut your_name).unwrap();
                your_name
            });

            template.fill_in_field("YOUR-FULL-NAME", &name);
            content = template.filled;
        } else {
            content = config::DEFAULT_CONFIG.to_owned();
        }



        for line in content.lines()
            .take_while(|l| !l.contains("-BREAK-"))
        {
            file.write_fmt(format_args!("{}\n", line))
                .expect("cannot write this line to the config file");
        }

        config_edit(editor);
    }
}

/// Command CONFIG --show
pub fn config_show(path: &str) -> Result<()> {
    println!("{}: {:#?}", path,
             CONFIG.get_to_string(&path));
    Ok(())
}

/// Command CONFIG --edit
fn config_edit(editor: Option<&str>) {
    let local = config::ConfigReader::path_home();
    if local.exists() {
        util::pass_to_command(editor, &[&CONFIG.path]);
    } else {
        error!("Cannot open {:?}, run `asciii config --init` to create it.", local)
    }
}

/// Command CONFIG --default
fn config_show_default() -> Result<()> {
    println!("{}", config::DEFAULT_CONFIG);
    Ok(())
}


/// Command DOC
pub fn doc() -> Result<()> {
    open::that(asciii::DOCUMENTATION_URL).unwrap(); //TODO
    //.and_then(|es| if !es.success() {Err("open-error".into())} else {Ok(())} )  ?
    Ok(())
}

/// Command VERSION
pub fn version() -> Result<()> {
    println!("{}", *asciii::VERSION);
    Ok(())
}

/// Command DUES
pub fn dues(matches: &ArgMatches) -> Result<()> {
    let dues = actions::dues();
    if let Ok(dues) = dues {
        println!("Open Payments: {}", dues.acc_sum_sold.postfix());
        println!("Open Wages:    {}", dues.acc_wages.postfix());
        if matches.is_present("wages") {
            for (employee, open_wages) in &dues.unpayed_employees {
                println!("{}:    {}", employee, open_wages.postfix());
            }
        }
    }
    Ok(())
}

// pub fn open_path(matches:&ArgMatches){path(matches, |path| {open::that(path).unwrap();})}
pub fn open_path(m: &ArgMatches) -> Result<()> {
    if m.is_present("search_term") {
        // let bill_type = infer_bill_type(m);
        // let template_name = "document";
        // let (search_terms, dir) = matches_to_search(m);
        unimplemented!()
    } else {
        path(m, |path| { Ok(open::that(path).map(|_| ())?) })?;
    }
    Ok(())
}

// TODO return result!
pub fn path<F>(m: &ArgMatches, action: F) -> Result<()>
    where F: Fn(&Path) -> Result<()>
{

    let path = CONFIG.get_str("path");
    let storage_path = CONFIG.get_str("dirs/storage");
    let templates_path = CONFIG.get_str("dirs/templates");
    let output_path = CONFIG.get_str("output_path");

    let exe = env::current_exe()?;

    Ok(if m.is_present("templates") {
        action(&PathBuf::from(path)
            .join(storage_path)
            .join(templates_path)
            )?
    }

    else if m.is_present("output") {
        action(&util::replace_home_tilde(Path::new(output_path)))?
    }

    else if m.is_present("bin") {
        action(
            exe.parent().unwrap()
               //.ok_or(Err("no parent".into()))? TODO
               )?
    }

    else {
        // default case
        let path = util::replace_home_tilde(Path::new(path)).join(storage_path);
        action(&path)?
    })
}

#[cfg(feature="shell")]
pub fn shell(_matches: &ArgMatches) -> Result<()> {
    shell::launch_shell()
}

#[cfg(not(feature="shell"))]
pub fn shell(_matches: &ArgMatches) -> Result<()> {
    bail!("Shell functionality not built-in with this release!");
}

