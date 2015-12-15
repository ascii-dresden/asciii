use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::collections::HashMap;

use slug;
use chrono::{Date, UTC};
use util::yaml::YamlError;

use templater::Templater;

const PROJECT_FILE_EXTENSION:&'static str = "yml";

pub type Year =  i32;
pub fn slugify(string:&str) -> String{ slug::slugify(string) }

#[derive(Debug)]
pub enum LuigiDir { Working, Archive(Year), Storage, Templates, All }

#[derive(Debug)]
pub enum LuigiSort { Date, Name, Index }

#[derive(Debug)]
pub enum LuigiError {
    DirectoryDoesNotExist(LuigiDir),
    NoProject,
    NoWorkingDir,
    ProjectFileExists,
    ProjectDirExists,
    StoragePathNotAbsolute,
    InvalidDirStructure,
    ParseError(YamlError),
    TemplateNotFound,
    Io(io::Error),
}

pub type LuigiResult<T> = Result<T, LuigiError>;

// All you need to make try!() fun again
impl From<io::Error>  for LuigiError {
    fn from(ioerror: io::Error) -> LuigiError{ LuigiError::Io(ioerror) }
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
}

impl Luigi {
    /// Inits luigi, does not check existence, yet. TODO
    pub fn new(storage:&Path, working:&str, archive:&str, template:&str) -> LuigiResult<Luigi> {
        Ok( Luigi{ // TODO check for the existence
            storage_dir:  storage.to_path_buf(),
            working_dir:  storage.join(working),
            archive_dir:  storage.join(archive),
            template_dir: storage.join(template),
        })
    }

    /// Getter for Luigi::storage_dir.
    pub fn storage_dir(&self) -> &Path{
        self.storage_dir.as_ref()
    }

    /// Generic Filesystem wrapper.
    fn list_path_content(&self, path:&Path) -> Vec<PathBuf> {
        let entries = fs::read_dir(path).unwrap();
        entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>()
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
        //TODO there must be something nicer than to_string(), like From<u32> for Path
        let archive = &self.archive_dir.join(year.to_string());

        if self.archive_dir.exists() && !archive.exists() {
            try!(fs::create_dir(archive));
        }
        Ok(archive.to_owned())
    }

    /// Produces a list of files in the `template_dir`
    /// TODO extension `.tyml` currently hardcoded
    pub fn list_template_files(&self) -> Vec<PathBuf> {
        self.list_path_content(&self.storage_dir.join(&self.template_dir))
            .iter()
            .filter(|p|p.extension().unwrap_or(OsStr::new("")) == OsStr::new("tyml"))
            .cloned().collect()
    }

    /// Produces a list of names of all template filess in the `template_dir`
    pub fn list_template_names(&self) -> Vec<String> {
        self.list_template_files().iter()
            .filter_map(|p|p.file_stem())
            .filter_map(|n|n.to_str())
            .map(|s|s.to_owned())
            .collect()
    }

    /// Returns the Path to the template file by the given name, maybe.
    pub fn get_template_file(&self, name:&str) -> LuigiResult<PathBuf> {
        self.list_template_files().iter()
            .filter(|f|f.file_stem().unwrap_or(&OsStr::new("")) == name)
            .cloned()
            .next().ok_or(LuigiError::TemplateNotFound)
    }

    /// Produces a list of paths to all archives in the `archive_dir`.
    /// An archive itself is a folder that contains project dirs,
    /// therefore it essentially has the same structure as the `working_dir`,
    /// with the difference, that the project folders may be prefixed with the projects index, e.g.
    /// an invoice number etc.
    pub fn list_archives(&self) -> Vec<PathBuf> {
        self.list_path_content(&self.storage_dir.join(&self.archive_dir))
    }

    /// Produces a list of years for which there is an archive.
    pub fn list_years(&self) -> Vec<Year> {
        let mut years : Vec<Year> =
            self.list_archives()
            .iter()
            .filter_map(|p| p.file_stem())
            .filter_map(|p| p.to_str())
            .filter_map(|year_str| year_str.parse::<Year>().ok())
            .collect();
        years.sort();
        years
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
        return Ok(project);
    }

    /// Moves a project folder from `/working` dir to `/archive/$year`.
    //TODO pub fn archive_project<T:LuigiProject>(&self, project:&T) {
    pub fn archive_project_with_year(&self, name:&str, year:Year) -> LuigiResult<PathBuf> {
        let slugged_name = slugify(name);
        let name_in_archive = format!("R000_{}", slugged_name); // TODO use actual index of project as Prefix

        let archive = try!(self.create_archive(year));
        let project_folder = try!(self.get_project_dir(
                name, LuigiDir::Working
                ).ok_or(LuigiError::NoProject));
        let target = archive.join(&name_in_archive);

        try!(fs::rename(&project_folder, &target));

        Ok(target)
    }

    /// Moves a project folder back from `/archive/$year` to `/working` dir.
    // TODO pub fn unarchive_project<T:LuigiProject>(&self, project:&T) {
    pub fn unarchive_project_with_year(&self, name:&str, year:Year) -> LuigiResult<PathBuf> {
        let slugged_name = slugify(name);
        if self.get_project_dir(&slugged_name, LuigiDir::Working).is_some(){
            return Err(LuigiError::ProjectFileExists);
        }
        let archive_dir = try!(self.get_project_dir(&slugged_name, LuigiDir::Archive(year)).ok_or(LuigiError::NoProject));
        let project_dir = self.working_dir.join(&slugged_name);
        if project_dir.exists() {
            // redundant? sure, but why not :D
            return Err(LuigiError::ProjectFileExists);
        }

        try!(fs::rename(&archive_dir, &project_dir));
        Ok(project_dir)
    }

    /// Tries to find a concrete Project.
    // TODO make this pathbuf a path
    pub fn get_project_dir(&self, name:&str, directory:LuigiDir) -> Option<PathBuf> {
        let slugged_name = slugify(name); // TODO wrap slugify in a function, so it can be adapted
        if let Some(path) = match directory{
            LuigiDir::Working => Some(self.working_dir.join(&slugged_name)),
            LuigiDir::Archive(year) => self.get_project_dir_archive(&name, year),
            _ => return None
        }{
            if path.exists(){
                return Some(path);
            }
        }
        None
    }

    /// Locates the project file inside a folder.
    ///
    /// This is the first file with the `PROJECT_FILE_EXTENSION` in the folder
    fn get_project_file(&self, directory:&Path) -> Option<PathBuf> {
        self.list_path_content(directory)
            .iter()
            .filter(|f|f.extension().unwrap_or(&OsStr::new("")) == PROJECT_FILE_EXTENSION)
            .next().map(|b|b.to_owned())
    }


    fn get_project_dir_archive(&self, name:&str, year:Year) -> Option<PathBuf> {
        for project_file in self.list_project_files(LuigiDir::Archive(year)).iter(){
            if project_file.ends_with(slugify(&name) + "."+ PROJECT_FILE_EXTENSION) {
                return project_file.parent().map(|p|p.to_owned());
            }
        }
        None
    }

    /// Produces a list of project folders.
    pub fn list_project_folders(&self, directory:LuigiDir) -> Vec<PathBuf> {
        match directory{
            LuigiDir::Working       => self.list_path_content(&self.working_dir),
            LuigiDir::Archive(year) => self.list_path_content(&self.archive_dir.join(year.to_string())),
            LuigiDir::All           => {
                let mut all:Vec<PathBuf> = Vec::new();
                for year in self.list_years(){
                    all.append(&mut self.list_path_content(&self.archive_dir.join(year.to_string())));
                }
                all.append(&mut self.list_path_content(&self.working_dir));
                all
            },
            _ => Vec::new()
        }
    }

    /// Produces a list of project files.
    /// **Deprecated**
    /// TODO make this a byproduct of a validation process
    pub fn list_broken_projects(&self, directory:LuigiDir) -> Vec<PathBuf>{
        self.list_project_folders(directory).iter()
            .filter(|dir| self.get_project_file(dir).is_none())
            .cloned()
            .collect()
    }

    /// Produces a list of project files.
    pub fn list_project_files(&self, directory:LuigiDir) -> Vec<PathBuf> {
        self.list_project_folders(directory).iter()
            .filter_map(|dir| self.get_project_file(dir))
            .collect()
    }
}

pub trait LuigiProject{
    /// creates in tempfile
    fn new(project_name:&str,template:&Path) -> LuigiResult<Self> where Self: Sized;
    fn name(&self) -> String;
    fn date(&self) -> Option<Date<UTC>>;
    fn index(&self) -> Option<String>;

    fn file_extension() -> &'static str {PROJECT_FILE_EXTENSION}
    fn file(&self) -> PathBuf; // Path to project file
    fn set_file(&mut self, new_file:&Path);
    fn dir(&self)  -> PathBuf{ self.file().parent().unwrap().to_owned() }
}

#[cfg(test)]
mod realworld {
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

        let templates = luigi.list_template_files();
        println!("{:#?}",templates);
        assert!(templates.len() == 3);
    }

    #[test]
    #[ignore]
    fn list_archives(){
        let (storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());

        let mut archives = luigi.list_archives();
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
        let (storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());

        let projects = luigi.list_project_files(LuigiDir::Archive(2015));
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
    use super::{Luigi, LuigiProject, LuigiError, LuigiDir};

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

        let templates = luigi.list_template_files();
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
        let test_p = TestProject::new("testproject", &template_path).unwrap();
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

        let mut archives = luigi.list_archives();
        let mut years = luigi.list_years();
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

        let templates = luigi.list_template_names();

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
    fn archive_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_template_names();
        for test_project in TEST_PROJECTS.iter() {
            // tested above
            let origin = luigi.create_project::<TestProject>( &test_project, &templates[0]).unwrap();

            // the actual tests
            assert!(luigi.archive_project_with_year(&test_project, 2015).is_ok());
            assert!(!origin.file().exists());

            assert!(luigi.get_project_dir(&test_project, LuigiDir::Working).is_none());
            assert!(luigi.get_project_dir(&test_project, LuigiDir::Archive(2015)).is_some());

            let false_origin = luigi.create_project::<TestProject>(&test_project, &templates[0]).unwrap();
            assert!(luigi.archive_project_with_year(&test_project, 2015).is_err());
        }
    }

    #[test]
    fn unarchive_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_template_names();
        for test_project in TEST_PROJECTS.iter() {
            // tested above
            let origin = luigi.create_project::<TestProject>( &test_project, &templates[0]).unwrap();
            luigi.archive_project_with_year(test_project, 2015).unwrap();

            // similarly looking archive
            luigi.create_project::<TestProject>( &test_project, &templates[0]).unwrap();
            luigi.archive_project_with_year(test_project, 2014).unwrap();

            // the actual tests
            assert_eq!(origin.dir(), luigi.unarchive_project_with_year(test_project,2015).unwrap());
            assert!(luigi.unarchive_project_with_year(test_project,2014).is_err());
        }
    }
}
