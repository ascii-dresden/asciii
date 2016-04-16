use std::path::{Path, PathBuf};

use chrono::{Date, UTC, Datelike};

use super::StorageResult;

pub trait Storable{
    /// opens a projectfile
    fn open(&Path) -> StorageResult<Self> where Self: Sized;

    /// creates in tempfile
    fn from_template(project_name:&str,template:&Path) -> StorageResult<Self> where Self: Sized;

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
    fn file_extension() -> &'static str {super::PROJECT_FILE_EXTENSION}

    /// Path to project file
    fn file(&self) -> PathBuf;

    /// Path to project folder
    fn dir(&self)  -> PathBuf{ self.file().parent().unwrap().to_owned() }

    fn matches_filter(&self, key: &str, val: &str) -> bool;
    fn matches_search(&self, term: &str) -> bool;
}
