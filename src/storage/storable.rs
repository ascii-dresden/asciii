//! Contains the `Storable` trait that storable projects must implement.
//!

use std::fmt;
use std::path::{Path, PathBuf};

use chrono::{Date, UTC, Datelike};
use term::color::Color;

use super::StorageResult; // status 

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

pub struct GitStorable<P:Storable+Sized>{
    pub inner:P,
    pub git_status:GitStatus
}

impl<P:Storable> GitStorable<P>{
    fn status(&self) -> GitStatus{ self.git_status }
}

impl<P:Storable> Storable for GitStorable<P>{
    fn open(path:&Path) -> StorageResult<Self> where Self: Sized {unimplemented!()}

    fn from_template(project_name:&str,template:&Path) -> StorageResult<Self> where Self: Sized {unimplemented!()}

    fn name(&self) -> String{self.inner.name()}
    fn date(&self) -> Option<Date<UTC>>{self.inner.date()}

    fn index(&self) -> Option<String>{self.inner.index()}
    fn prefix(&self) -> Option<String>{self.inner.prefix()}

    fn set_file(&mut self, new_file:&Path){self.inner.set_file(new_file)}

    fn file(&self) -> PathBuf{self.inner.file()}

    fn matches_filter(&self, key: &str, val: &str) -> bool{self.inner.matches_filter(key,val)}
    fn matches_search(&self, term: &str) -> bool{self.inner.matches_search(term)}
}


#[derive(Copy,Clone)]
pub enum GitStatus{
    Unchanged,
    ProjectAdded,
    ProjectRemoved,
    FileAdded,
    FileChanged,
    FileRemoved,
    Erroneous
}

impl GitStatus{
    pub fn to_color(&self) -> Color {
        use term::color;
        use term::color::Color;
        match *self{
            GitStatus::Unchanged      => color::WHITE,
            GitStatus::ProjectAdded   => color::BLUE,
            GitStatus::ProjectRemoved => color::RED,
            GitStatus::FileAdded      => color::GREEN,
            GitStatus::FileChanged    => color::RED,
            GitStatus::FileRemoved    => color::RED,
            GitStatus::Erroneous      => color::RED,
        }
    }
}

impl fmt::Display for GitStatus{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match *self{
            GitStatus::Unchanged      => write!(f, ""),
            GitStatus::ProjectAdded   => write!(f, "+"),
            GitStatus::ProjectRemoved => write!(f, "-"),
            GitStatus::FileAdded      => write!(f, "+"),
            GitStatus::FileChanged    => write!(f, "~"),
            GitStatus::FileRemoved    => write!(f, "-"),
            GitStatus::Erroneous      => write!(f, "X"),
        }
    }
}


