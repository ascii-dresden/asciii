#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(dead_code)]
use std::fs;
use std::fs::{File,PathExt,DirEntry};
use std::path::{Path, PathBuf};

use chrono::{DateTime, UTC};

//TODO: nice Errors
//TODO: add logging

#[derive(Debug)]
pub enum LuigiDirectory { Working, Archive(i32), Storage, Template }

#[derive(Debug)]
pub enum LuigiSort { Date, Name, Index }

#[derive(Debug)]
pub enum LuigiError
{
    DirectoryDoesNotExist(LuigiDirectory),
    ProjectFileExists,
    TemplateDoesNotExist,
    StoragePathNotAbsolute,
    NoDirectory,
    UnableToCreate
}

#[derive(Debug)]
pub struct Luigi
{
    pub storage_dir:  PathBuf,
    pub working_dir:  PathBuf,
    pub archive_dir:  PathBuf,
    pub template_dir: PathBuf,
}

impl Luigi
{
    pub fn new(storage:&Path, working:&str, archive:&str, template:&str) -> Result<Luigi, LuigiError>
    {
        // TODO check for the existence
        Ok( Luigi{
            storage_dir:  storage.to_path_buf(),
            working_dir:  storage.join(working),
            archive_dir:  storage.join(archive),
            template_dir: storage.join(template),
        })
    }

    /// creates the basic dir structure inside the storage directory
    ///
    ///└── storage
    ///    ├── archive
    ///    ├── templates
    ///    └── working
    pub fn create_dirs(&self) -> Result<(), LuigiError>
    {
        if !self.storage_dir.as_path().is_absolute() { return Err(LuigiError::StoragePathNotAbsolute) }

        if !self.storage_dir.exists()  { fs::create_dir(&self.storage_dir).unwrap();  }
        if !self.working_dir.exists()  { fs::create_dir(&self.working_dir).unwrap();  }
        if !self.archive_dir.exists()  { fs::create_dir(&self.archive_dir).unwrap();  }
        if !self.template_dir.exists() { fs::create_dir(&self.template_dir).unwrap(); }

        Ok(())
    }

    /// Creates an archive for a certain year.
    /// This is a subdirectory under the archive directory.
    ///└── storage
    ///    ├── archive
    ///    |   ├── 2001
    ///    ...
    pub fn create_archive(&self, year:i32)
        //-> Result<(), LuigiError>
    {
        assert!(self.archive_dir.exists());
        let archive = &self.archive_dir.join(year.to_string()); /*TODO there must be something nicer than to_string()*/

        if self.archive_dir.exists() && !archive.exists() {
            fs::create_dir(archive).unwrap();
        }

    }

    fn list_path_content(&self, path:&PathBuf )
        -> Vec<PathBuf>
    {
        let entries = fs::read_dir(path).unwrap();
        entries.map(|entry|{
            entry.unwrap().path()
        }).collect::<Vec<PathBuf>>()
    }

    pub fn list_templates(&self) -> Vec<PathBuf> {
        self.list_path_content(&self.storage_dir.join(&self.template_dir))
    }

    pub fn list_archives(&self) -> Vec<PathBuf> {
        self.list_path_content(&self.storage_dir.join(&self.template_dir))
    }

    pub fn list_projects(&self, directory:LuigiDirectory) -> Option<Vec<PathBuf>>
    {
        match directory{
            LuigiDirectory::Working =>
                Some(self.list_path_content(&self.working_dir)),
            LuigiDirectory::Archive(year) =>
                Some(self.list_path_content(&self.archive_dir.join(year.to_string()))),
            _ => None
        }
    }
}

trait LuigiProject
{
    fn open<T: LuigiProject>(path:Path) -> Result<T,LuigiError>;
    fn index(&self) -> String;
    fn name(&self) -> String;
    fn date(&self) -> DateTime<UTC>;
    fn path(&self) -> PathBuf;
    fn file_extension() -> String;
}



#[cfg(test)]
mod test
{
    use std::path::{Path,PathBuf};
    use std::fs::{File,PathExt};
    use util;
    use util::ls;

    extern crate tempdir;
    pub use self::tempdir::TempDir;
    pub use super::{Luigi,LuigiError};

    fn setup() -> (TempDir, PathBuf, Luigi)
    {
        let dir = TempDir::new_in(Path::new("."),"luigi_test").unwrap();
        let storage_path = dir.path().join("storage");
        let luigi = Luigi::new(&storage_path, "working", "archive", "templates").unwrap();
        (dir, storage_path, luigi)
    }

    #[test]
    fn create_dirs() {
        let (dir , storage_path, luigi) = setup();
        luigi.create_dirs().unwrap();

        // all my paths exist
        assert!(storage_path.exists());
        assert!(storage_path.join("working").exists());
        assert!(storage_path.join("archive").exists());
        assert!(storage_path.join("templates").exists());

        // calling it again does not cause problems
        assert!(luigi.create_dirs().is_ok());

        // all my paths still exist
        assert!(storage_path.exists());
        assert!(storage_path.join("working").exists());
        assert!(storage_path.join("archive").exists());
        assert!(storage_path.join("templates").exists());

        util::ls(&dir.path().to_string_lossy());
        //freeze(); // for manual checking
    }

    #[test]
    fn list_templates(){
        let (_dir , storage_path, luigi) = setup();
        luigi.create_dirs().unwrap();

        // all my paths exist
        assert!(storage_path.exists());
        assert!(storage_path.join("working").exists());
        assert!(storage_path.join("archive").exists());
        assert!(storage_path.join("templates").exists());

        util::copy_template(storage_path.join("templates"));

        let templates = luigi.list_templates();
        println!("{:#?}",templates);
        assert!(templates.len() == 2);

        //util::freeze();
    }

    #[test]
    fn create_archive(){
        let (_dir , storage_path, luigi) = setup();
        assert!(luigi.create_dirs().is_ok());
        luigi.create_archive(2001);
        luigi.create_archive(2002);
        luigi.create_archive(2002); // should this fail?
        assert!(storage_path.join("archive").join("2001").exists());
        assert!(storage_path.join("archive").join("2002").exists());
        util::ls(&_dir.path().to_string_lossy());

    }

    #[test]
    fn list_archives(){
        unimplemented!();
    }

    //#[test]
    //fn create_project(){
    //    let (_dir , storage_path, luigi) = setup();
    //    unimplemented!();
    //}
}


