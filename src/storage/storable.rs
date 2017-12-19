//! Contains the `Storable` trait that storable projects must implement.
//!

use std::{fs,io};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use chrono::{Date, Utc, Datelike};

use super::StorageResult;
use super::repo::GitStatus;

pub type FilePath = Path;
pub type FolderPath = Path;

pub type FilePathBuf = PathBuf;
pub type FolderPathBuf = PathBuf;

pub trait Storable: Send+Sync{
    /// opens a projectfolder
    fn open_folder(&FolderPath) -> StorageResult<Self> where Self: Sized;
    fn open_file(&FilePath) -> StorageResult<Self> where Self: Sized;

    /// creates in tempfile
    fn from_template(project_name:&str,template:&Path, data:&HashMap<&str, String>) -> StorageResult<Self> where Self: Sized;

    /// For file names
    fn ident(&self) -> String{ self.dir().file_stem().and_then(|s|s.to_str()).unwrap().to_owned() }

    fn short_desc(&self) -> String;
    fn modified_date(&self) -> Option<Date<Utc>>;
    fn year(&self) -> Option<i32>{ self.modified_date().map(|d|d.year()) }

    /// Deletes the project if the passed in closure returns `true`
    fn delete_project_dir_if<F>(&self, confirmed:F) -> io::Result<()>
        where F: Fn()->bool
    {
        let folder = self.dir();
        if confirmed(){
            debug!("$ rm {}", folder.display());
            fs::remove_dir_all(&folder)
        } else {Ok(())}
    }

    /// For sorting
    fn index(&self) -> Option<String>;

    /// For archiving
    fn prefix(&self) -> Option<String>;

    /// Sets the project File
    fn set_file(&mut self, new_file:&Path);

    /// Tell a project its own git status after opening
    ///
    /// This depoends on the feature `git_statuses`
    fn set_git_status(&mut self, GitStatus){}

    /// Ask a project for its gitstatus
    ///
    /// This depoends on the feature `git_statuses`
    fn get_git_status(&self) -> GitStatus{GitStatus::Unknown}

    /// Main Projectfile extension
    fn file_extension() -> String {String::from("PROJECT")}

    /// Path to project file
    fn file(&self) -> FilePathBuf;

    /// Filename as fallback
    fn file_name(&self) -> String {
        self.file().file_name().expect("filename ended in ..").to_string_lossy().into()
    }

    /// Path to project folder
    fn dir(&self)  -> FolderPathBuf { self.file().parent().unwrap().to_owned() }

    fn matches_filter(&self, key: &str, val: &str) -> bool;
    fn matches_search(&self, term: &str) -> bool;

    fn is_ready_for_archive(&self) -> bool;
}
