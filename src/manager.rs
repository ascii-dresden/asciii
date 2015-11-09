#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::fs;
use std::io;
use std::fs::{File,Metadata,DirEntry,ReadDir};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use slug;
use chrono::{DateTime, UTC};

//TODO: add logging
//TODO: remove asserts, is_ok()s and unwrap()s, stupid :D

const PROJECT_FILE_EXTENSION:&'static str = "yml";

pub type Year =  u32;
pub fn slugify(string:&str) -> String{ slug::slugify(string) }

#[derive(Debug)]
pub enum LuigiDir { Working, Archive(Year), Storage, Template }

#[derive(Debug)]
pub enum LuigiSort { Date, Name, Index }

#[derive(Debug)]
pub enum LuigiError {
    DirectoryDoesNotExist(LuigiDir),
    NoProject,
    NoWorkingDir,
    ProjectFileExists,
    StoragePathNotAbsolute,
    InvalidDirStructure,
    Io(io::Error),
    //UnableToCreate,
    //TemplateDoesNotExist,
}

// All you need to make try!() fun again
impl From<io::Error>  for LuigiError {
    fn from(ioerror:io::Error) -> LuigiError{ LuigiError::Io(ioerror) }
}

#[derive(Debug)]
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
    pub fn new(storage:&Path, working:&str, archive:&str, template:&str) -> Result<Luigi, LuigiError> {
        Ok( Luigi{ // TODO check for the existence
            storage_dir:  storage.to_path_buf(),
            working_dir:  storage.join(working),
            archive_dir:  storage.join(archive),
            template_dir: storage.join(template),
        })
    }

    fn list_path_content(&self, path:&Path) -> Vec<PathBuf> {
        let entries = fs::read_dir(path).unwrap();
        entries.map(|entry|{
            entry.unwrap().path()
        }).collect::<Vec<PathBuf>>()
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
    pub fn create_dirs(&self) -> Result<(), LuigiError> {
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
    /// </pre>
    pub fn create_archive(&self, year:Year) -> Result<PathBuf, LuigiError> {
        assert!(self.archive_dir.exists());
        //TODO there must be something nicer than to_string(), like From<u32> for Path
        let archive = &self.archive_dir.join(year.to_string());

        if self.archive_dir.exists() && !archive.exists() {
            try!(fs::create_dir(archive));
        }
        Ok(archive.to_owned())
    }

    /// Produces a list of paths to all template filess in the `template_dir`
    pub fn list_templates(&self) -> Vec<PathBuf> {
        self.list_path_content(&self.storage_dir.join(&self.template_dir))
    }

    /// Produces a list of paths to all archives in the `archive_dir`.
    /// An archive itself is a folder that contains project dirs,
    /// therefore it essentially has the same structure as the `working_dir`,
    /// with the difference, that the project folders may be prefixed with the projects index, e.g.
    /// an invoice number etc.
    pub fn list_archives(&self) -> Vec<PathBuf> {
        self.list_path_content(&self.storage_dir.join(&self.archive_dir))
    }

    /// Fills a template file and stores it in the working directory,
    /// in a new project directory according to it's name.
    //TODO take a T:LuigiProject
    pub fn create_project(&self, project:&str, template:&Path) -> Result<PathBuf, LuigiError>{
        if !self.working_dir.exists(){ return Err(LuigiError::NoWorkingDir)}; // funny syntax
        let slugged_name = slugify(project);
        let project_dir = self.working_dir.join(&slugged_name);
        let project_file = project_dir.join(&(slugged_name + "." + PROJECT_FILE_EXTENSION));

        if self.working_dir.exists() && !project_dir.exists() {
            //TODO replace copy with LuigiProejct::new(...).store(path)
            try!(fs::create_dir(&project_dir));
            try!(fs::copy(&template, &project_file));
        }
        Ok(project_dir.to_owned())
    }

    fn move_folder(folder:&Path, target:&Path) -> Result<(), LuigiError>{
        // chreate target dir
        try!(fs::create_dir(&target));

        // TODO there must be a less manual way, perhaps through some crate
        // copy all the files over
        for file in try!(fs::read_dir(&folder)){
            let path = try!(file).path();
            try!(fs::copy( &path, &target.join(
                            try!(path.file_name().ok_or(LuigiError::InvalidDirStructure))
                        )
                    ));
        }

        // remove old location
        try!(fs::remove_dir_all(&folder));
        Ok(())
    }

    //TODO pub fn archive_project<T:LuigiProject>(&self, project:&T) {
    pub fn archive_project_with_year(&self, name:&str, year:Year) -> Result<PathBuf, LuigiError> {
        let slugged_name = slugify(name);
        let name_in_archive = format!("R000_{}", slugged_name); // TODO use actual index of project as Prefix

        let archive = try!(self.create_archive(year));
        let project_folder = try!(self.get_project_dir(
                name, LuigiDir::Working
                ).ok_or(LuigiError::NoProject));
        let target = archive.join(&name_in_archive);

        try!(Luigi::move_folder(&project_folder, &target));

        Ok(target)
    }

    // TODO pub fn unarchive_project<T:LuigiProject>(&self, project:&T) {
    pub fn unarchive_project_with_year(&self, name:&str, year:Year) -> Result<PathBuf, LuigiError> {
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

        try!(Luigi::move_folder(&archive_dir, &project_dir));
        Ok(project_dir)
    }

    // TODO make this pathbuf a path
    pub fn get_project_dir(&self, name:&str, directory:LuigiDir) -> Option<PathBuf> {
        let slugged_name = slugify(name); // TODO wrap slugify in a function, so it can be adapted
        let project_dir = &self.working_dir.join(&slugged_name);
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

    fn get_project_dir_archive(&self, name:&str, year:Year) -> Option<PathBuf> {
        for project_file in self.list_project_files(LuigiDir::Archive(year)).iter(){
            if project_file.ends_with(slugify(&name) + "."+ PROJECT_FILE_EXTENSION) {
                return project_file.parent().map(|p|p.to_owned());
            }
        }
        None
    }

    // TODO make this private
    pub fn get_project_file(&self, directory:&Path) -> Option<PathBuf> {
        self.list_path_content(directory)
            .iter()
            .filter(|f|f.extension().unwrap_or(&OsStr::new("")) == PROJECT_FILE_EXTENSION)
            .next().map(|b|b.to_owned())
    }

    pub fn list_projects(&self, directory:LuigiDir) -> Vec<PathBuf> {
        match directory{
            LuigiDir::Working => self.list_path_content(&self.working_dir),
            LuigiDir::Archive(year) => self.list_path_content(&self.archive_dir.join(year.to_string())),
            _ => Vec::new()
        }
    }

    pub fn list_project_files(&self, directory:LuigiDir) -> Vec<PathBuf> {
        self.list_projects(directory).iter()
            .map(|dir| self.get_project_file(dir))
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
            .collect()
    }
}

pub trait LuigiProject {
    fn new<T>(name:&str) -> T where T:LuigiProject;
    //fn open<T: LuigiProject>(path:Path) -> Result<T,LuigiError>;
    fn index(&self) -> String;
    fn name(&self) -> String;
    //fn date(&self) -> DateTime<UTC>;
    //n path(&self) -> PathBuf;
    fn file_extension() -> String;
}


#[cfg(test)]
mod realworld {
    use std::path::{Path,PathBuf};
    use std::fs::{File,Metadata};
    use util;
    use util::ls;

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
    fn list_templates(){
        let (storage_path, luigi) = setup();
        //luigi.create_dirs().unwrap();
        assert_existens(&storage_path);

        let templates = luigi.list_templates();
        println!("{:#?}",templates);
        assert!(templates.len() == 3);
    }

    #[test]
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
    fn list_projects(){
        let (storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());

        let projects = luigi.list_project_files(LuigiDir::Archive(2015));
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
    use std::fs::{File,Metadata};
    use slug;
    use util;
    use util::ls;

    extern crate tempdir;
    pub use self::tempdir::TempDir;
    pub use super::{Luigi,
                    LuigiProject,
                    LuigiError,
                    LuigiDir};

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
    fn list_templates(){
        let (_dir , storage_path, luigi) = setup();
        luigi.create_dirs().unwrap();
        assert_existens(&storage_path);

        copy_template(storage_path.join("templates"));

        let templates = luigi.list_templates();
        println!("{:#?}",templates);
        assert!(templates.len() == 2);

        ////util::freeze();
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
        archives.sort();
        println!("ARCHIVES\n{:#?}", archives);

        assert!(archives[0].ends_with("1999"));
        assert!(archives[1].ends_with("2001"));
        assert!(archives[2].ends_with("2002"));
    }

    #[test]
    fn create_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_templates();

        for test_project in TEST_PROJECTS.iter() {
            let target_path = luigi.create_project( &test_project, &templates[0]).unwrap();
            assert!(target_path.exists());

            let target_file = target_path.join(&(slug::slugify(test_project) + "." + super::PROJECT_FILE_EXTENSION));
            assert!(target_file.exists());
            assert_eq!(target_file, luigi.get_project_file(&target_path).unwrap());

            let project = luigi.get_project_dir(test_project, LuigiDir::Working);
            assert!(project.unwrap().exists());

            let project = luigi.get_project_dir(test_project, LuigiDir::Working);
            assert_eq!(project.unwrap(), target_path);
        }
    }

    #[test]
    fn archive_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_templates();
        for test_project in TEST_PROJECTS.iter() {
            // tested above
            let origin = luigi.create_project( &test_project, &templates[0]).unwrap();

            // the actual tests
            assert!(luigi.archive_project_with_year(&test_project, 2015).is_ok());
            assert!(!origin.exists());

            assert!(luigi.get_project_dir(&test_project, LuigiDir::Working).is_none());
            assert!(luigi.get_project_dir(&test_project, LuigiDir::Archive(2015)).is_some());

            let false_origin = luigi.create_project(&test_project, &templates[0]).unwrap();
            assert!(luigi.archive_project_with_year(&test_project, 2015).is_err());
        }
    }

    #[test]
    fn unarchive_project(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        assert_existens(&storage_path);
        copy_template(storage_path.join("templates"));

        let templates = luigi.list_templates();
        for test_project in TEST_PROJECTS.iter() {
            // tested above
            let origin = luigi.create_project( &test_project, &templates[0]).unwrap();
            luigi.archive_project_with_year(test_project, 2015).unwrap();

            // similarly looking archive
            luigi.create_project( &test_project, &templates[0]).unwrap();
            luigi.archive_project_with_year(test_project, 2014).unwrap();

            // the actual tests
            assert_eq!(origin, luigi.unarchive_project_with_year(test_project,2015).unwrap());
            assert!(luigi.unarchive_project_with_year(test_project,2014).is_err());
        }
    }
}
