use std::path::{Path,PathBuf};
use std::ffi::OsStr;
use std::{env,fs};
use std::io::Write;
use std::collections::HashMap;

use open;
use clap::ArgMatches;
use chrono::*;

use asciii;
use asciii::CONFIG;
use asciii::config;
use asciii::util;
use asciii::BillType;
use asciii::actions;
use asciii::storage::*;
use asciii::templater::Templater;
use asciii::project::Project;
use asciii::project::ComputedField;
use asciii::actions::{setup_luigi, setup_luigi_with_git};

#[cfg(feature="document_export")]
use rustc_serialize::json::ToJson;

use asciii::print;
use asciii::print::{ListConfig, ListMode};
//simple_rows, verbose_rows,
//path_rows, dynamic_rows,
//print_projects,print_csv};

use super::{execute,fail};

// TODO refactor this into actions module and actual, short subcommands

/// Create NEW Project
//#[deprecated(note="move to asciii::actions")]
pub fn new(matches:&ArgMatches){
    let project_name     = matches.value_of("name").expect("You did not pass a \"Name\"!");
    let editor           = CONFIG.get("user/editor").and_then(|e|e.as_str());

    let template_name = matches.value_of("template")
        .or( CONFIG.get("template").unwrap().as_str())
        .unwrap();

    let edit = !matches.is_present("don't edit");
    let luigi = execute(setup_luigi);

    let mut fill_data:HashMap<&str, String> = HashMap::new();

    if let Some(description) = matches.value_of("description"){
        debug!("Filling in DESCRIPTION");
        fill_data.insert("DESCRIPTION", description.to_owned());
    }

    if let Some(date) = matches.value_of("date"){
        debug!("Filling in DATE-EVENT");
        fill_data.insert("DATE-EVENT", date.to_owned());
    }

    if let Some(time) = matches.value_of("time"){
        debug!("Filling in TIME-START");
        fill_data.insert("TIME-START", time.to_owned());
    }

    if let Some(time_end) = matches.value_of("time_end"){
        debug!("Filling in TIME-END");
        fill_data.insert("TIME-END", time_end.to_owned());
    }

    if let Some(manager) = matches.value_of("manager"){
        debug!("Filling in MANAGER");
        fill_data.insert("MANAGER", manager.to_owned());
    }

    let project = execute(|| luigi.create_project(project_name, template_name, &fill_data));
    let project_file = project.file();
    if edit {
        util::pass_to_command(&editor, &[project_file]);
    }
}

//#[deprecated(note="move to impl ListMode and then to asciii::actions")]
fn decide_mode(simple:bool, verbose:bool, paths:bool,nothing:bool, csv:bool) -> ListMode{
    if csv{     ListMode::Csv }
    else if nothing{ ListMode::Nothing }
    else if paths{   ListMode::Paths }
    else {
        match (simple, verbose, CONFIG.get_bool("list/verbose")){
            (false, true,  _   ) => {debug!("-v overwrites config"); ListMode::Verbose },
            (false,    _, true ) => {debug!("-v from config");ListMode::Verbose},
                          _      => {debug!("simple mode");ListMode::Simple},
        }
    }
}

fn matches_to_search<'a>(matches:&'a ArgMatches) -> (Vec<&'a str>, StorageDir) {
    let search_terms = matches
        .values_of("search_term")
        .map(|v|v.collect::<Vec<&str>>())
        .unwrap_or_else(Vec::new);

    debug!("matches_to_search: --archive={:?}", matches.value_of("archive"));


    let dir =
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
        else { StorageDir::Working };

    (search_terms, dir)
}

/// Produces a list of paths.
/// This is more general than `with_projects`, as this includes templates too.
pub fn matches_to_paths(matches:&ArgMatches, luigi:&Storage<Project>) -> Vec<PathBuf>{
    let search_terms = matches
        .values_of("search_term")
        .map(|v|v.collect::<Vec<&str>>())
        .unwrap_or_else(Vec::new);

    if matches.is_present("template"){
        super::execute(||luigi.list_template_files())
            .into_iter()
            .filter(|f|{
                let stem = f.file_stem()
                    .and_then(OsStr::to_str)
                    .unwrap_or("");
                search_terms.contains(&stem)
            })
        .collect::<Vec<_>>()
    } else {
        let dir = if let Some(archive) = matches.value_of("archive"){
            StorageDir::Archive(archive.parse::<i32>().unwrap())
        } else {
            StorageDir::Working
        };

        super::execute(||luigi.search_projects_any(dir, &search_terms))
            .iter()
            .map(|project|project.dir())
            .collect::<Vec<_>>()

    }
}

/// Command LIST
pub fn list(matches:&ArgMatches){
    if matches.is_present("templates"){
        list_templates();
    } else if matches.is_present("years"){
        list_years();
    } else if matches.is_present("computed_fields"){
        list_computed_fields();
    }

    else {
        let list_mode = decide_mode(
            matches.is_present("simple"),
            matches.is_present("verbose"),
            matches.is_present("paths"),
            matches.is_present("nothing"),
            matches.is_present("csv")
            );

        let extra_details = matches.values_of("details").map(|v|v.collect::<Vec<&str>>());
        let config_details = CONFIG.get_strs("list/extra_details");

        let mut list_config = ListConfig{
            sort_by:   matches.value_of("sort")
                              .unwrap_or_else(||CONFIG.get_str("list/sort")
                                              .expect("Faulty config: field list/sort does not contain a string value")
                                             ),
            mode:      list_mode,
            details:   extra_details.or(config_details),
            filter_by: matches.values_of("filter").map(|v|v.collect::<Vec<&str>>()),
            show_errors: matches.is_present("errors"),

            ..Default::default()
        };

        if matches.is_present("colors"){ list_config.use_colors = true; }
        if matches.is_present("no-colors"){ list_config.use_colors = false; }

        // list archive of year `archive`
        let dir =
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
    let luigi = if CONFIG.get_bool("list/gitstatus"){
        execute(setup_luigi_with_git)
    } else {
        execute(setup_luigi)
    };
    debug!("listing projects: {}", luigi.working_dir().display());

    let mut projects = execute(||luigi.open_projects(dir));

    // filtering, can you read this
    if let Some(ref filters) = list_config.filter_by{
        projects.filter_by_all(filters);
    }

    // sorting
    match list_config.sort_by {
        "manager" => projects.sort_by(|pa,pb| pa.manager().cmp( &pb.manager())),
        "date"    => projects.sort_by(|pa,pb| pa.date().cmp( &pb.date())),
        "name"    => projects.sort_by(|pa,pb| pa.name().cmp( &pb.name())),
        "index"   => projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned()))), // TODO rename to ident
        _         => projects.sort_by(|pa,pb| pa.index().unwrap_or("zzzz".to_owned()).cmp( &pb.index().unwrap_or("zzzz".to_owned()))),
    }

    // fit screen
    let wide_enough = true;

    if !wide_enough && list_config.mode != ListMode::Csv { // TODO room for improvement
        print::print_projects(print::simple_rows(&projects, list_config));
    } else {
        debug!("list_mode: {:?}", list_config.mode );
        match list_config.mode{
            ListMode::Csv     => print::print_csv(&projects),
            ListMode::Paths   => print::print_projects(print::path_rows(&projects, list_config)),
            ListMode::Simple  => print::print_projects(print::simple_rows(&projects, list_config)),
            ListMode::Verbose => print::print_projects(print::verbose_rows(&projects,list_config)),
            ListMode::Nothing => print::print_projects(print::dynamic_rows(&projects,list_config)),
        }
    }
}

/// Command LIST --broken
fn list_broken_projects(dir:StorageDir){
    let luigi = execute(setup_luigi);
    let invalid_files = execute(||luigi.list_project_files(dir));
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
    let luigi = execute(setup_luigi);

    for name in execute(||luigi.list_template_names()){
        println!("{}", name);
    }
}

/// Command LIST --years
fn list_years(){
    let luigi = execute(setup_luigi);
    let years = execute(||luigi.list_years());
    println!("{:?}", years);
}

/// Command LIST --virt
fn list_computed_fields(){
    println!("{:?}", ComputedField::iter_variant_names().filter(|v|*v!="Invalid").collect::<Vec<&str>>());
}


/// Command CSV
pub fn csv(matches:&ArgMatches){
    use chrono::{Local,Datelike};
    let year = matches.value_of("year")
        .and_then(|y|y.parse::<i32>().ok())
        .unwrap_or(Local::now().year());

    debug!("asciii csv --year {}", year);
    let csv = execute(||actions::csv(year));
    println!("{}", csv);
}


/// Command EDIT
pub fn edit(matches:&ArgMatches) {
    let search_term = matches.value_of("search_term").unwrap();
    let search_terms = matches.values_of("search_term").unwrap().collect::<Vec<&str>>();

    let editor = matches.value_of("editor")
        .or( CONFIG.get("user/editor").and_then(|e|e.as_str()));

    if matches.is_present("template"){
        edit_template(search_term, &editor);

    } else if let Some(archive) = matches.value_of("archive"){
        let archive = archive.parse::<i32>().unwrap();
        edit_projects(StorageDir::Archive(archive), &search_terms, &editor);
    } else {
        edit_projects(StorageDir::Working, &search_terms, &editor);
    }
}

fn edit_projects(dir:StorageDir, search_terms:&[&str], editor:&Option<&str>){
    let luigi = execute(setup_luigi);
    let mut all_projects= Vec::new();
    for search_term in search_terms{
        let mut paths = execute(||luigi.search_projects(dir, search_term));
        if paths.is_empty(){
            //println!{"Nothing found for {:?}", search_term}
        } else {
            all_projects.append(&mut paths);
        }
    }

    if all_projects.is_empty(){
        fail(format!("Nothing found for {:?}", search_terms));
    } else {
        let all_paths = all_projects.iter().map(|p|p.file()).collect::<Vec<PathBuf>>();
        util::pass_to_command(&editor, &all_paths);
    }
}

/// Command EDIT --template
fn edit_template(name:&str, editor:&Option<&str>){
    let luigi = execute(setup_luigi);
    let template_paths = execute(||luigi.list_template_files())
        .into_iter() // drain?
        .filter(|f|f.file_stem() .unwrap_or_else(||OsStr::new("")) == name)
        .collect::<Vec<PathBuf>>();
    util::pass_to_command(editor, &template_paths);
}

/// Command SET
pub fn set(m:&ArgMatches){
    let field = m.value_of("field name").unwrap().chars().flat_map(|c|c.to_uppercase()).collect::<String>();
    let value = m.value_of("field value").unwrap();
    let (search_terms, dir) = matches_to_search(m);

    execute(||actions::with_projects(dir, &search_terms, |project| {
        println!("{}: {}", project.name(), project.empty_fields().join(", "));
        if !project.empty_fields().contains(&field) {
            return Err(format!("{:?} was not found in {}", field, project.name()).into());
        }
        if util::really(&format!("do you want to set the field {} in {:?} [y|N]", field, project.name())) {
            project.replace_field(&field,&value).map_err(|e|e.into())
        } else {
            Err("Don't want to".into())
        }
    }))
}


#[cfg(feature="document_export")]
fn infer_bill_type(m:&ArgMatches) -> Option<BillType> {
    match (m.is_present("offer"),m.is_present("invoice")) {
        (true,true)  => unreachable!("this should have been prevented by clap-rs"),
        (true,false) => Some(BillType::Offer),
        (false,true) => Some(BillType::Invoice),
        (false,false) => None
    }
}

/// Command SHOW
pub fn show(m:&ArgMatches){
    let (search_terms, dir) = matches_to_search(m);

    let bill_type = match (m.is_present("offer"),m.is_present("invoice")) {
        (true,true)  => unreachable!("this should have been prevented by clap-rs"),
        (true,false) => BillType::Offer,
      //(false,true) => BillType::Invoice,
        _            => BillType::Invoice, //TODO be inteligent here ( use date )
    };


    if m.is_present("files"){
        actions::simple_with_projects(dir, &search_terms,
                      |p| {
                          println!("{}: ", p.dir().display());
                          for entry in fs::read_dir(p.dir()).unwrap(){
                              println!("  {}", entry.unwrap().path().display())
                          }
                      }
                     );
    } else if let Some(detail) = m.value_of("detail"){
        show_detail(dir, &search_terms, detail);
    } else if m.is_present("empty fields"){ show_empty_fields(dir, search_terms.as_slice())
    } else if m.is_present("errors"){ show_errors(dir, search_terms.as_slice())
    } else if m.is_present("dump"){ dump_yaml(dir, search_terms.as_slice())
    } else if m.is_present("json"){ show_json(dir, search_terms.as_slice())
    } else if m.is_present("csv"){  show_csv( dir, search_terms.as_slice());
    } else if m.is_present("template"){ show_template(search_terms[0]);
    } else { actions::simple_with_projects(dir, search_terms.as_slice(), |p|print::show_details(p,&bill_type)) }
}

fn dump_yaml(dir:StorageDir, search_terms:&[&str]){
    actions::simple_with_projects(dir, &search_terms, |p| println!("{}", p.dump_yaml()));
}

fn show_errors(dir:StorageDir, search_terms:&[&str]){
    actions::simple_with_projects(dir, &search_terms, |p| println!("{}: {}", p.name(), p.is_ready_for_archive().err().unwrap_or_else(Vec::new).join(", ")));
}

fn show_empty_fields(dir:StorageDir, search_terms:&[&str]){
    actions::simple_with_projects(dir, &search_terms, |p| println!("{}: {}", p.name(), p.empty_fields().join(", ")));
}


#[cfg(feature="document_export")]
fn show_json(dir:StorageDir, search_terms:&[&str]){
    actions::simple_with_projects(dir, &search_terms, |p| println!("{}", p.to_json()));
}

fn show_detail(dir:StorageDir, search_terms:&[&str], detail:&str){
    actions::simple_with_projects(dir, &search_terms, |p| println!("{}", p.get(detail).unwrap_or_else(||String::from("Nothing found"))));
}

fn show_csv(dir:StorageDir, search_terms:&[&str]){
    actions::simple_with_projects(dir, &search_terms, |p| println!("{}", execute(||p.to_csv(&BillType::Invoice))));
}

#[cfg(not(feature="document_export"))]
fn show_json(_:StorageDir, _:&[&str]){
    error!("feature temporarily disabled")
}

/// Command SPEC
/// TODO make this not panic :D
/// TODO move this to `spec::all_the_things`
pub fn spec(_:&ArgMatches){
    execute(||actions::spec())
}

/// Command MAKE
#[cfg(feature="document_export")]
pub fn make(m:&ArgMatches){
    let template_name = m.value_of("template").unwrap_or("document");
    let bill_type = infer_bill_type(m);
    let (search_terms, dir) = matches_to_search(m);

    debug!("make {t}({s}/{d:?}, invoice={i:?})",
    d = dir,
    s = search_terms[0],
    t = template_name,
    i = bill_type
    );

    execute(|| actions::projects_to_doc(dir,
                                        search_terms[0],
                                        template_name,
                                        &bill_type,
                                        m.is_present("dry-run"),
                                        m.is_present("force")
                                       )
           );
}



/// Command DELETE
pub fn delete(m:&ArgMatches){
    let (search_terms, dir) = matches_to_search(m);
    execute(||actions::delete_project_confirmation(dir, &search_terms));
}

#[cfg(not(feature="document_export"))]
pub fn make(_:&ArgMatches){
    error!("Make functionality not built-in with this release!");
    unimplemented!();
}







/// Command SHOW --template
fn show_template(name:&str){
    let luigi = execute(setup_luigi);
    let template = execute(||luigi.get_template_file(name));
    let templater = execute(||Templater::from_file(&template));
    println!("{:#?}", templater.list_keywords());
}

fn add_to_git(paths:&[PathBuf]) {
    let luigi = execute(||setup_luigi_with_git());
    if !paths.is_empty() {
        if let Some(repo) = luigi.repository{
            util::exit(repo.add(&paths));
        }
    }
}

/// TODO make this be have like `edit`, taking multiple names
pub fn archive(matches:&ArgMatches){
    let search_terms = matches.values_of("search terms").unwrap().collect::<Vec<_>>();
    let year = matches.value_of("year").and_then(|s|s.parse::<i32>().ok());
    trace!("archive({:?},{:?})",search_terms, year);
    let moved_files = execute(|| actions::archive_projects(&search_terms, year, matches.is_present("force")));
    add_to_git(&moved_files);
}

pub fn unarchive(matches:&ArgMatches){
    let year = matches.value_of("year").unwrap();
    let year = year.parse::<i32>().unwrap_or_else(|e|panic!("can't parse year {:?}, {:?}", year, e));
    let search_terms = matches.values_of("name").unwrap().collect::<Vec<_>>();
    let moved_files = execute(||actions::unarchive_projects(year, &search_terms));
    add_to_git(&moved_files);
}

pub fn config(matches:&ArgMatches){
    if let Some(path) = matches.value_of("show"){
        config_show(path);
    }
    if matches.is_present("location") {
        println!("config location: {:?}", config::ConfigReader::path_home())
    }

    else if matches.is_present("init") {
        let local = config::ConfigReader::path_home();
        println!("config location: {:?}", local);
        if local.exists(){
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
            .or( CONFIG.get("user/editor").and_then(|e|e.as_str()));
        config_edit(&editor);
    }

    else if matches.is_present("default"){ config_show_default(); }
}



/// Command CONFIG --show
pub fn config_show(path:&str){
    println!("{}: {:#?}", path, CONFIG.get_to_string(&path)
             .unwrap_or_else(||format!("{} not set", path)));
}

/// Command CONFIG --edit
fn config_edit(editor:&Option<&str>){
    let local = config::ConfigReader::path_home();
    if local.exists(){
        util::pass_to_command(editor, &[&CONFIG.path]);
    } else {
        error!("Cannot open {:?}, run `asciii config --init` to create it.", local)
    }
}

/// Command CONFIG --default
fn config_show_default(){
    println!("{}", config::DEFAULT_CONFIG);
}


/// Command DOC
pub fn doc(){
    open::that(asciii::DOCUMENTATION_URL).unwrap();
}

/// Command VERSION
pub fn version(){
    println!("{}", *asciii::VERSION);
}

/// Command DUES
pub fn dues(matches:&ArgMatches){
    let dues = if matches.is_present("wages") {
        actions::open_wages()
    } else {
        actions::open_payments()
    };
    if let Ok(dues) = dues {
        println!("{}", dues.postfix());
    }
}

pub fn show_path(matches:&ArgMatches){path(matches, |path| println!("{}", path.display()))}

//pub fn open_path(matches:&ArgMatches){path(matches, |path| {open::that(path).unwrap();})}
pub fn open_path(m:&ArgMatches){
    if m.is_present("search_term") {
        //let bill_type = infer_bill_type(m);
        //let template_name = "document";
        //let (search_terms, dir) = matches_to_search(m);
        unimplemented!()
    } else {
        path(m, |path| {open::that(path).unwrap();})
    }
}

pub fn path<F:Fn(&Path)>(m:&ArgMatches, action:F){
    let path = CONFIG.get_str("path").expect("Faulty config: field output_path does not contain a string value");
    let storage_path = CONFIG.get_str("dirs/storage").expect("Faulty config: field output_path does not contain a string value");
    let templates_path = CONFIG.get_str("dirs/templates").expect("Faulty config: field output_path does not contain a string value");
    let output_path = CONFIG.get_str("output_path").expect("Faulty config: field output_path does not contain a string value");

    let exe = env::current_exe().unwrap();

    if m.is_present("templates") {
        action(&PathBuf::from(path)
               .join(storage_path)
               .join(templates_path
                    ));
    }
    else if m.is_present("output") {
        action(
            &util::replace_home_tilde(
                Path::new(output_path)
                ));
    }
    else if m.is_present("bin") {
        action(exe.parent().unwrap());
    }
    else { // default case
        let path = util::replace_home_tilde(Path::new(path))
            .join( storage_path );
        action(&path);
    }
}



/// Command LOG
pub fn git_log(){
    let luigi = execute(||setup_luigi_with_git());
    let repo = luigi.repository.unwrap();
    util::exit(repo.log()) // FIXME this does not behave right
}

/// Command STATUS
pub fn git_status(){
    let luigi = execute(||setup_luigi_with_git());
    let repo = luigi.repository.unwrap();
    util::exit(repo.status()) // FIXME this does not behave right
}

/// Command COMMIT
pub fn git_commit(){
    let luigi = execute(||setup_luigi_with_git());
    let repo = luigi.repository.unwrap();
    util::exit(repo.commit())
}

/// Command REMOTE
/// exact replica of `git remote -v`
#[cfg(not(feature="git_statuses"))]
pub fn git_remote(){
    let luigi = execute(||setup_luigi_with_git());
    luigi.repository.unwrap().remote();
}

/// Command REMOTE
/// exact replica of `git remote -v`
#[cfg(feature="git_statuses")]
pub fn git_remote(){
    let luigi = execute(||setup_luigi_with_git());
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
    let luigi = execute(||setup_luigi_with_git());
    let paths = matches_to_paths(matches, &luigi);
    let repo = luigi.repository.unwrap();
    util::exit(repo.add(&paths));
}


/// Command DIFF
pub fn git_diff(matches:&ArgMatches){
    let luigi = execute(||setup_luigi_with_git());
    let paths = matches_to_paths(matches, &luigi);
    let repo = luigi.repository.unwrap();
    util::exit(repo.diff(&paths))
}

/// Command PULL
pub fn git_pull(matches:&ArgMatches){
    let luigi = execute(||setup_luigi_with_git());
    let repo = luigi.repository.unwrap();

    if matches.is_present("rebase"){
        util::exit(repo.pull_rebase())
    } else {
        util::exit(repo.pull())
    }
}

/// Command PUSH
pub fn git_push(){
    let luigi = execute(||setup_luigi_with_git());
    let repo = luigi.repository.unwrap();
    util::exit(repo.push())
}

/// Command STASH
pub fn git_stash(){
    let luigi = execute(||setup_luigi_with_git());
    let repo = luigi.repository.unwrap();
    util::exit(repo.stash())
}

/// Command CLEANUP
pub fn git_cleanup(matches:&ArgMatches){
    let luigi = execute(||setup_luigi_with_git());
    let paths = matches_to_paths(matches, &luigi);
    let repo = luigi.repository.unwrap();
    // TODO implement `.and()` for exitstatus

    if util::really(&format!("Do you really want to reset any changes you made to:\n {:?}\n[y|N]", paths))
    {
        let checkout_status = repo.checkout(&paths);
        if checkout_status.success() {
            util::exit(repo.clean(&paths))
        } else {
            debug!("clean checkout was no success");
            util::exit(checkout_status)
        }
    }
}

/// Command DIFF
pub fn git_stash_pop(){
    let luigi = execute(||setup_luigi_with_git());
    let repo = luigi.repository.unwrap();
    util::exit(repo.stash_pop())
}
