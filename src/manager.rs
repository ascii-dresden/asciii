#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::collections::HashMap;

use slug;
use chrono::{Date, UTC, Datelike};
use git2::Error as GitError;

use repo;
use repo::{GitStatus,Repository};
use util::yaml::YamlError;

use templater::Templater;

const PROJECT_FILE_EXTENSION:&'static str = "yml";
const TEMPLATE_FILE_EXTENSION:&'static str = "tyml";

pub type Year =  i32;
pub type LuigiResult<T> = Result<T, LuigiError>;

// All you need to make try!() fun again
impl From<io::Error>  for LuigiError {
    fn from(io_error: io::Error) -> LuigiError{ LuigiError::Io(io_error) }
}

impl From<GitError>  for LuigiError {
    fn from(git_error: GitError) -> LuigiError{ LuigiError::Git(git_error) }
}

fn slugify(string:&str) -> String{ slug::slugify(string) }

#[derive(Debug)]
pub enum LuigiDir { Working, Archive(Year), Storage, Templates, All }

#[derive(Debug)]
pub enum LuigiSort { Date, Name, Index }

#[derive(Debug)]
pub enum LuigiError {
    DirectoryDoesNotExist(LuigiDir),
    BadChoice,
    NoProject,
    NoWorkingDir,
    ProjectFileExists,
    ProjectDirExists,
    ProjectDoesNotExist,
    StoragePathNotAbsolute,
    InvalidDirStructure,
    ParseError(YamlError),
    TemplateNotFound,
    Git(GitError),
    Io(io::Error),
}

pub trait LuigiProject{
    /// creates in tempfile
    fn new(project_name:&str,template:&Path) -> LuigiResult<Self> where Self: Sized;

    /// For file names
    fn ident(&self) -> String{ self.dir().file_stem().and_then(|s|s.to_str()).unwrap().to_owned() }

    fn name(&self) -> String;
    fn date(&self) -> Option<Date<UTC>>;
    fn year(&self) -> Option<i32>{ self.date().map(|d|d.year()) }

    /// For sorting
    fn index(&self) -> Option<String>;
    /// For archiving
    fn prefix(&self) -> Option<String>;

    fn set_file(&mut self, new_file:&Path);
    fn file_extension() -> &'static str {PROJECT_FILE_EXTENSION}

    /// Path to project file
    fn file(&self) -> PathBuf;

    /// Path to project folder
    fn dir(&self)  -> PathBuf{ self.file().parent().unwrap().to_owned() }
}

// TODO rely more on IoError, it has most of what you need
/// Does all the internal plumbing.
///
/// This includes:
///
/// * listing project folders and files
/// * listing templates
/// * archiving and unarchiving projects
/// * git interaction ( not yet )
pub struct Luigi {
    /// Root of the entire Structure.
    storage_dir:  PathBuf,
    /// Place for project directories.
    working_dir:  PathBuf,
    /// Place for archive directories (*e.g. `2015/`*) each containing project directories.
    archive_dir:  PathBuf,
    /// Place for template files.
    template_dir: PathBuf,

    pub repository: Option<Repository>
}

use std::fmt;
impl fmt::Debug for Luigi{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "Luigi: storage  = {storage:?}
                          working  = {working:?}
                          archive  = {archive:?}
                          template = {template:?}",
               storage  = self.storage_dir,
               working  = self.working_dir,
               archive  = self.archive_dir,
               template = self.template_dir,
               )
    }
}

impl Luigi {
    /// Inits luigi, does not check existence, yet. TODO
    pub fn new<P: AsRef<Path>>(storage:P, working:&str, archive:&str, template:&str) -> LuigiResult<Luigi> {
        let storage = storage.as_ref();
        if storage.is_absolute(){
            Ok( Luigi{
                storage_dir:  storage.to_path_buf(),
                working_dir:  storage.join(working),
                archive_dir:  storage.join(archive),
                template_dir: storage.join(template),
                repository: None,
            })
        } else {
            Err(LuigiError::StoragePathNotAbsolute)
        }
    }

    /// Inits luigi with git capabilities.
    pub fn new_with_git<P: AsRef<Path>>(storage:P, working:&str, archive:&str, template:&str) -> LuigiResult<Self> {
        Ok( Luigi{
            repository: Some(try!(Repository::new(storage.as_ref()))),
            .. try!{Self::new(storage,working,archive,template)}
        })
    }

    /// Getter for Luigi::storage_dir.
    pub fn storage_dir(&self) -> &Path{
        self.storage_dir.as_ref()
    }

    /// Generic Filesystem wrapper.
    fn list_path_content(&self, path:&Path) -> LuigiResult<Vec<PathBuf>> {
        let entries = try!(fs::read_dir(path))
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>();
        Ok(entries)
    }

    /// Creates the basic dir structure inside the storage directory.
    ///
    ///<pre>
    ///└── storage
    ///    ├── archive
    ///    ├── templates
    ///    └── working
    ///</pre>
    /// If the directories already exist as expected, that's fine
    /// TODO ought to fail when storage_dir already contains directories that do not correspond
    /// with the names given in this setup.
    pub fn create_dirs(&self) -> LuigiResult<()> {
        if !self.storage_dir.as_path().is_absolute() { return Err(LuigiError::StoragePathNotAbsolute) }

        if !self.storage_dir.exists()  { try!(fs::create_dir(&self.storage_dir));  }
        if !self.working_dir.exists()  { try!(fs::create_dir(&self.working_dir));  }
        if !self.archive_dir.exists()  { try!(fs::create_dir(&self.archive_dir));  }
        if !self.template_dir.exists() { try!(fs::create_dir(&self.template_dir)); }

        Ok(())
    }

    /// Creates an archive for a certain year.
    /// This is a subdirectory under the archive directory.
    ///<pre>
    ///└── storage
    ///    ├── archive
    ///        ├── 2001
    ///    ...
    ///</pre>
    pub fn create_archive(&self, year:Year) -> LuigiResult<PathBuf> {
        assert!(self.archive_dir.exists());
        let archive = &self.archive_dir.join(year.to_string());

        if self.archive_dir.exists() && !archive.exists() {
            try!(fs::create_dir(archive));
        }
        Ok(archive.to_owned())
    }

    /// Produces a list of files in the `template_dir`
    pub fn list_template_files(&self) -> LuigiResult<Vec<PathBuf>> {
        let template_files =
        try!(self.list_path_content(&self.template_dir))
            .iter()
            .filter(|p|p.extension()
                        .unwrap_or_else(|| OsStr::new("")) == OsStr::new(TEMPLATE_FILE_EXTENSION)
                        )
            .cloned().collect();
        Ok(template_files)
    }

    /// Produces a list of names of all template filess in the `template_dir`
    pub fn list_template_names(&self) -> LuigiResult<Vec<String>> {
        let template_names =
        try!(self.list_template_files()).iter()
            .filter_map(|p|p.file_stem())
            .filter_map(|n|n.to_str())
            .map(|s|s.to_owned())
            .collect();
        Ok(template_names)
    }

    /// Returns the Path to the template file by the given name, maybe.
    pub fn get_template_file(&self, name:&str) -> LuigiResult<PathBuf> {
        try!(self.list_template_files()).iter()
            .filter(|f|f.file_stem().unwrap_or(&OsStr::new("")) == name)
            .cloned()
            .nth(0).ok_or(LuigiError::TemplateNotFound)
    }

    /// Produces a list of paths to all archives in the `archive_dir`.
    /// An archive itself is a folder that contains project dirs,
    /// therefore it essentially has the same structure as the `working_dir`,
    /// with the difference, that the project folders may be prefixed with the projects index, e.g.
    /// an invoice number etc.
    pub fn list_archives(&self) -> LuigiResult<Vec<PathBuf>> {
        self.list_path_content(&self.archive_dir)
    }

    /// Produces a list of years for which there is an archive.
    pub fn list_years(&self) -> LuigiResult<Vec<Year>> {
        let mut years : Vec<Year> =
            try!(self.list_archives())
            .iter()
            .filter_map(|p| p.file_stem())
            .filter_map(|p| p.to_str())
            .filter_map(|year_str| year_str.parse::<Year>().ok())
            .collect();
        years.sort();
        Ok(years)
    }

    /// Takes a template file and stores it in the working directory,
    /// in a new project directory according to it's name.
    pub fn create_project<P:LuigiProject>(&self, project_name:&str, template_name:&str) -> LuigiResult<P> {
        if !self.working_dir.exists(){ return Err(LuigiError::NoWorkingDir)}; // funny syntax
        let slugged_name = slugify(&project_name);
        let project_dir  = self.working_dir.join(&slugged_name);
        if project_dir.exists() { return Err(LuigiError::ProjectDirExists); }

        let target_file  = project_dir
            .join(&(slugged_name + "." + PROJECT_FILE_EXTENSION));

        let template_path = try!(self.get_template_file(template_name));
        let mut project = try!(P::new(&project_name, &template_path));

        // TODO test for unreplaced template keywords
        try!(fs::create_dir(&project_dir));
        try!(fs::copy(project.file(), &target_file));
        project.set_file(&target_file);

        Ok(project)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    pub fn archive_project_by_name(&self, name:&str, year:Year, prefix:Option<String>) -> LuigiResult<PathBuf> {
        let slugged_name = slugify(name);
        let name_in_archive = match prefix{
            Some(prefix) => format!("{}_{}", prefix, slugged_name),
                    None => slugged_name
        };

        let archive = try!(self.create_archive(year));
        let project_folder = try!(self.get_project_dir(name, LuigiDir::Working));
        let target = archive.join(&name_in_archive);

        try!(fs::rename(&project_folder, &target));

        Ok(target)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    /// Also adds the project.prefix() to the folder name.
    ///<pre>
    ///└── storage
    ///    ├── archive
    ///        ├── 2001
    ///            ├── R0815_Birthdayparty
    ///    ...
    ///</pre>
    // TODO write extra tests
    pub fn archive_project<T:LuigiProject>(&self, project:&T, year:Year) -> LuigiResult<PathBuf> {
        let name_in_archive = match project.prefix(){
            Some(prefix) => format!("{}_{}", prefix, project.ident()),
                    None =>  project.ident()
        };

        let archive = try!(self.create_archive(year));
        let project_folder = project.dir();
        let target = archive.join(&name_in_archive);

        try!(fs::rename(&project_folder, &target));

        Ok(target)
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    pub fn unarchive_project_file(&self, archived_file:&Path) -> LuigiResult<PathBuf> {
        let archived_dir = if archived_file.is_file() { try!(archived_file.parent().ok_or(LuigiError::InvalidDirStructure)) } else {archived_file};

        // has to be in archive_dir
        let child_of_archive = archived_file.starts_with(&self.archive_dir);

        // must not be the archive_dir
        let archive_itself =  archived_dir == self.archive_dir;

        // must be in a dir that parses into a year
        let parent_is_num =  archived_dir.parent()
            .and_then(|p| p.file_stem())
            .and_then(|p| p.to_str())
            .map(|s| s.parse::<i32>().is_ok() )
            .unwrap_or(false);

        let name = try!(self.get_project_name(archived_dir));
        let target = self.working_dir.join(&name);
        if target.exists() { return Err(LuigiError::ProjectFileExists); }

        if child_of_archive && !archive_itself && parent_is_num{
            try!(fs::rename(&archived_dir, &target));
        }else{
            println!("not cool");
            return Err(LuigiError::InvalidDirStructure);
        };

        Ok(target.to_owned())
    }

    /// Moves a project folder back from `/archive/$year` to `/working` dir.
    // TODO pub fn unarchive_project<T:LuigiProject>(&self, project:&T) {
    pub fn unarchive_project_by_name(&self, name:&str, year:Year) -> LuigiResult<PathBuf> {
        let slugged_name = slugify(name);
        if self.get_project_dir(&slugged_name, LuigiDir::Working).is_ok(){
            return Err(LuigiError::ProjectFileExists);
        }
        let archive_dir = try!(self.get_project_dir(&slugged_name, LuigiDir::Archive(year)));
        let project_dir = self.working_dir.join(&slugged_name);
        if project_dir.exists() { return Err(LuigiError::ProjectFileExists); }

        try!(fs::rename(&archive_dir, &project_dir));
        Ok(project_dir)
    }

    /// Matches LuigiDir's content against a term and returns matching project files.
    pub fn search_projects(&self, dir:LuigiDir, search_term:&str) -> LuigiResult<Vec<PathBuf>> {
        let projects: Vec<PathBuf> = try!(self.list_project_files(dir))
            .iter()
            .filter(|path| path.to_str().unwrap_or("??").to_lowercase().contains(&search_term.to_lowercase()))
            .cloned()
            .collect()
            ;
        Ok(projects)
    }

    /// Tries to find a concrete Project.
    pub fn get_project_dir(&self, name:&str, directory:LuigiDir) -> LuigiResult<PathBuf> {
        let slugged_name = slugify(name);
        if let Ok(path) = match directory{
            LuigiDir::Working => Ok(self.working_dir.join(&slugged_name)),
            LuigiDir::Archive(year) => self.get_project_dir_from_archive(&name, year),
            _ => return Err(LuigiError::BadChoice)
        }{
            if path.exists(){
                return Ok(path);
            }
        }
        Err(LuigiError::ProjectDoesNotExist)
    }

    /// Locates the project file inside a folder.
    ///
    /// This is the first file with the `PROJECT_FILE_EXTENSION` in the folder
    fn get_project_file(&self, directory:&Path) -> LuigiResult<PathBuf> {
        try!(self.list_path_content(directory)).iter()
            .filter(|f|f.extension().unwrap_or(&OsStr::new("")) == PROJECT_FILE_EXTENSION)
            .nth(0).map(|b|b.to_owned())
            .ok_or(LuigiError::ProjectDoesNotExist)
    }

    fn get_project_name(&self, directory:&Path) -> LuigiResult<String> {
        let path = try!(self.get_project_file(directory));
        Ok(path.file_stem().unwrap().to_str().unwrap().to_owned())
    }

    fn get_project_dir_from_archive(&self, name:&str, year:Year) -> LuigiResult<PathBuf> {
        for project_file in &try!(self.list_project_files(LuigiDir::Archive(year))){
            if project_file.ends_with(slugify(&name) + "."+ PROJECT_FILE_EXTENSION) {
                return project_file.parent().map(|p|p.to_owned()).ok_or(LuigiError::NoProject);
            }
        }
        Err(LuigiError::ProjectDoesNotExist)
    }

    /// Produces a list of project folders.
    pub fn list_project_folders(&self, directory:LuigiDir) -> LuigiResult<Vec<PathBuf>> {
        match directory{
            LuigiDir::Working       => self.list_path_content(&self.working_dir),
            LuigiDir::Archive(year) => self.list_path_content(&self.archive_dir.join(year.to_string())),
            LuigiDir::All           => {
                let mut all:Vec<PathBuf> = Vec::new();
                for year in try!(self.list_years()){
                    all.append(&mut try!(self.list_path_content(&self.archive_dir.join(year.to_string()))));
                }
                all.append(&mut try!(self.list_path_content(&self.working_dir)));
                Ok(all)
            },
            _ => Err(LuigiError::BadChoice)
        }
    }

    /// Produces a list of empty project folders.
    pub fn list_empty_project_dirs(&self, directory:LuigiDir) -> LuigiResult<Vec<PathBuf>> {
        let projects = try!(self.list_project_folders(directory)).iter()
            .filter(|dir| self.get_project_file(dir).is_err())
            .cloned()
            .collect();
        Ok(projects)
    }

    /// Produces a list of project files.
    pub fn list_project_files(&self, directory:LuigiDir) -> LuigiResult<Vec<PathBuf>> {
        let projects = try!(self.list_project_folders(directory)).iter()
            .filter_map(|dir| self.get_project_file(dir).ok())
            .collect();
        Ok(projects)
    }

}

#[cfg(test)]
pub mod realworld {
    use std::path::{Path,PathBuf};

    pub use super::{Luigi,LuigiError,LuigiDir};

    const STORAGE:&'static str = "/home/hendrik/ascii/caterings";

    fn setup() -> (PathBuf, Luigi) {
        let storage_path = PathBuf::from(STORAGE);
        let luigi = Luigi::new(&storage_path, "working", "archive", "templates").unwrap();
        (storage_path, luigi)
    }

    fn assert_existens(storage_path:&Path) {
        assert!(storage_path.exists()
            &&  storage_path.join("working").exists()
            &&  storage_path.join("archive").exists()
            &&  storage_path.join("templates").exists());
    }

    #[test]
    #[ignore]
    fn list_template_files(){
        let (storage_path, luigi) = setup();
        //luigi.create_dirs().unwrap();
        assert_existens(&storage_path);

        let templates = luigi.list_template_files().unwrap();
        println!("{:#?}",templates);
        assert!(templates.len() == 3);
    }

    #[test]
    #[ignore]
    fn list_archives(){
        let (_storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());

        let mut archives = luigi.list_archives().unwrap();
        archives.sort();
        println!("ARCHIVES\n{:#?}", archives);

        assert!(archives[0].ends_with("2012"));
        assert!(archives[1].ends_with("2013"));
        assert!(archives[2].ends_with("2014"));
        assert!(archives[3].ends_with("2015"));
    }

    #[test]
    #[ignore]
    fn list_project_folders(){
        let (_storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());

        //let projects = luigi.list_project_files(LuigiDir::Archive(2015));
        let projects = luigi.list_project_files(LuigiDir::Working);
        println!("Projects");
        for p in projects{
            println!("{:#?}", p);
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path,PathBuf};
    use std::fs;
    use std::fs::File;
    use slug;
    use util;
    use util::ls;

    use chrono::*;
    use tempdir::TempDir;
    use super::{Luigi, LuigiProject, LuigiResult, LuigiError, LuigiDir};

    pub struct TestProject {
        file_path: PathBuf,
        temp_dir: Option<TempDir>
    }

    impl LuigiProject for TestProject{
        // creates in tempfile
        fn new(project_name:&str,template:&Path) -> LuigiResult<Self> where Self: Sized {
            // generates a temp file
            let temp_dir  = TempDir::new_in(".",&project_name).unwrap();
            let temp_file = temp_dir.path().join(project_name);

            // just copy over template
            try!(fs::copy(template, &temp_file));

            // project now lives in the temp_file
            Ok(TestProject{
                file_path: temp_file,
                temp_dir: Some(temp_dir),
            })
        }
        fn name(&self) -> String{ self.file().file_stem().unwrap().to_str().unwrap().to_owned() }
        fn date(&self) -> Option<Date<UTC>>{ Some(UTC::today()) }
        fn file(&self) -> PathBuf{ self.file_path.to_owned() }
        fn set_file(&mut self, new_file:&Path){ self.file_path = new_file.to_owned(); }
        fn index(&self) -> Option<String>{ Some("ZZ99".into()) }
        fn prefix(&self) -> Option<String>{ self.index() }
    }


    // TODO implement failing cases
    const TEST_PROJECTS:[&'static str;4] = [
        "test1", "test2",
        "foobar", "ich schreibe viel zu längliche projektnamen!",
    ];


    fn setup() -> (TempDir, PathBuf, Luigi) {
        let dir = TempDir::new_in(Path::new("."),"luigi_test").unwrap();
        let storage_path = dir.path().join("storage");
        let luigi = Luigi::new(&storage_path, "working", "archive", "templates").unwrap();
        (dir, storage_path, luigi)
    }

    fn assert_existens(storage_path:&Path) {
        assert!(storage_path.exists()
            &&  storage_path.join("working").exists()
            &&  storage_path.join("archive").exists()
            &&  storage_path.join("templates").exists());
    }

    fn copy_template(target:PathBuf) {
        fs::copy("./templates/default.tyml", target.join("template1.tyml")).unwrap();
        fs::copy("./templates/default.tyml", target.join("template2.tyml")).unwrap();
    }

    #[test]
    fn create_dirs() {
        let (dir , storage_path, luigi) = setup();
        luigi.create_dirs().unwrap();
        assert_existens(&storage_path);

        // calling it again does not cause problems
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);

        util::ls(&dir.path().to_string_lossy());
    }

    #[test]
    fn list_template_files(){
        let (_dir , storage_path, luigi) = setup();
        luigi.create_dirs().unwrap();
        assert_existens(&storage_path);

        copy_template(storage_path.join("templates"));

        let templates = luigi.list_template_files().unwrap();
        println!("{:#?}",templates);
        assert!(templates.len() == 2);

        ////util::freeze();
    }

    #[test]
    fn create_test_project(){
        let (_dir , storage_path, luigi) = setup();
        luigi.create_dirs().unwrap();
        assert_existens(&storage_path);

        copy_template(storage_path.join("templates"));
        let template_path = luigi.get_template_file("template1").unwrap();
        TestProject::new("testproject", &template_path).unwrap();
    }

    #[test]
    fn create_archive(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        luigi.create_archive(2001).unwrap();
        luigi.create_archive(2002).unwrap();
        luigi.create_archive(2002).unwrap(); // should this fail?
        assert!(storage_path.join("archive").join("2001").exists());
        assert!(storage_path.join("archive").join("2002").exists());
        util::ls(&_dir.path().to_string_lossy());
    }

    #[test]
    fn list_archives(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        luigi.create_archive(2001).unwrap();
        luigi.create_archive(2002).unwrap();
        luigi.create_archive(1999).unwrap();
        util::ls(&_dir.path().to_string_lossy());

        let mut archives = luigi.list_archives().unwrap();
        let mut years = luigi.list_years().unwrap();
        archives.sort();
        years.sort();
        println!("ARCHIVES\n{:#?}", archives);

        assert!(archives[0].ends_with("1999"));
        assert!(archives[0].is_dir());
        assert!(archives[1].ends_with("2001"));
        assert!(archives[1].is_dir());
        assert!(archives[2].ends_with("2002"));
        assert!(archives[2].is_dir());

        println!("ARCHIVES\n{:#?}", years);
        assert_eq!(years[0], 1999);
        assert_eq!(years[1], 2001);
        assert_eq!(years[2], 2002);
    }

    #[test]
    fn create_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_template_names().unwrap();

        for test_project in TEST_PROJECTS.iter() {
            let project     = luigi.create_project::<TestProject>(&test_project, &templates[0]).unwrap();
            let target_file = project.file();
            let target_path = target_file.parent().unwrap();
            assert!(target_path.exists());
            assert!(target_file.exists());
            util::ls(&target_path.display().to_string());
            assert_eq!(target_file, luigi.get_project_file(&target_path).unwrap());

            let project_dir = luigi.get_project_dir(test_project, LuigiDir::Working);
            assert!(project_dir.unwrap().exists());

            let project_dir = luigi.get_project_dir(test_project, LuigiDir::Working);
            assert_eq!(project_dir.unwrap(), target_path);
        }
    }

    #[test]
    fn archive_project_by_name(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_template_names().unwrap();
        for test_project in TEST_PROJECTS.iter() {
            // tested above
            let origin = luigi.create_project::<TestProject>( &test_project, &templates[0]).unwrap();

            // the actual tests
            assert!(luigi.archive_project_by_name(&test_project, 2015, None).is_ok());
            assert!(!origin.file().exists());

            assert!(luigi.get_project_dir(&test_project, LuigiDir::Working).is_err());
            assert!(luigi.get_project_dir(&test_project, LuigiDir::Archive(2015)).is_ok());

            //let false_origin = luigi.create_project::<TestProject>(&test_project, &templates[0]).unwrap();
            assert!(luigi.archive_project_by_name(&test_project, 2015, None).is_err());
        }
    }

    #[test]
    fn archive_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let year = UTC::today().year();

        let templates = luigi.list_template_names().unwrap();
        for test_project_name in TEST_PROJECTS.iter() {
            // tested above
            let project = luigi.create_project::<TestProject>( &test_project_name, &templates[0]).unwrap();

            // Before archiving
            assert!(project.file().exists());
            assert!(luigi.get_project_dir(&test_project_name, LuigiDir::Working).is_ok());

            // ARCHIVING
            assert!(luigi.archive_project(&project, project.year().unwrap()).is_ok());

            // After archiving
            assert!(!project.file().exists());
            assert!(luigi.get_project_dir(&test_project_name, LuigiDir::Working).is_err());
            assert!(luigi.get_project_dir(&test_project_name, LuigiDir::Archive(year)).is_ok());

            assert!(luigi.archive_project(&project, year).is_err());
        }
    }

    #[test]
    fn unarchive_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_template_names().unwrap();
        for test_project in TEST_PROJECTS.iter() {
            // tested above
            let origin = luigi.create_project::<TestProject>( &test_project, &templates[0]).unwrap();
            luigi.archive_project_by_name(test_project, 2015, None).unwrap();

            // similarly looking archive
            luigi.create_project::<TestProject>( &test_project, &templates[0]).unwrap();
            luigi.archive_project_by_name(test_project, 2014, None).unwrap();

            // the actual tests
            assert_eq!(origin.dir(), luigi.unarchive_project_by_name(test_project,2015).unwrap());
            assert!(luigi.unarchive_project_by_name(test_project,2014).is_err());
        }
    }
    #[test]
    fn unarchive_project2(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_template_names().unwrap();
        for test_project in TEST_PROJECTS.iter() {
            let origin = luigi.create_project::<TestProject>( &test_project, &templates[0]).unwrap();
            luigi.archive_project_by_name(test_project, 2015, None).unwrap();
        }

        for year in luigi.list_years().unwrap(){
            println!("{:?}", year);
            for proj in luigi.list_project_files(LuigiDir::Archive(year)).unwrap() {
                assert!(luigi.unarchive_project_file(&proj).is_ok());
                assert!(luigi.unarchive_project_file(&proj).is_err());
            }
        }
    }
}
