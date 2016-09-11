//! Manages file structure of templates, working directory and archives.
//!
//! This module takes care of project file management.
//!
//! Your ordinary file structure would look something like this:
//!
//! ```bash
//! # root dir
//! ├── working
//! │   └── Project1
//! │       └── Project1.yml
//! ├── archive
//! │   ├── 2013
//! │   └── 2014
//! │       └── R036_Project3
//! │           ├── Project3.yml
//! │           └── R036 Project3 2014-10-08.tex
//! ...
//! ```
//!


use std::fs;
use std::path::{Path, PathBuf};
use std::marker::PhantomData;

static TEMPLATE_FILE_EXTENSION:&'static str = "tyml";

/// Year = `i32`
pub type Year =  i32;

/// Result returned by Storage
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)] mod test;
#[cfg(test)] mod realworld;

mod project_list;
pub use self::project_list::ProjectList;
pub mod repo;
pub mod error;
pub use self::error::StorageError;
pub mod storable;
pub use self::storable::Storable;

mod storage;

// TODO rely more on IoError, it has most of what you need
/// Manages project file storage.
///
/// This includes:
///
/// * keeping current projects in a working directory
/// * listing project folders and files
/// * listing templates
/// * archiving and unarchiving projects
/// * git interaction
pub struct Storage<L:Storable> {
    /// Root of the entire Structure.
    root:  PathBuf,
    /// Place for project directories.
    working:  PathBuf,
    /// Place for archive directories (*e.g. `2015/`*) each containing project directories.
    archive:  PathBuf,
    /// Place for template files.
    templates: PathBuf,

    project_type: PhantomData<L>,

    pub repository: Option<repo::Repository>
}

/// Used to identify what directory you are talking about.
#[derive(Debug,Clone,Copy)]
#[allow(dead_code)]
pub enum StorageDir {
    /// Describes exclusively the working directory.
    Working,
    /// Describes exclusively one year's archive.
    Archive(Year),
    /// Describes archive of year and working directory,
    /// if this year is still current.
    Year(Year),
    /// Parent of `Working`, `Archive` and `Templates`.
    Root,
    /// Place to store templates.
    Templates,
    /// `Archive` and `Working` directory, not `Templates`.
    All
}

/// Basically `ls`, returns a list of paths.
pub fn list_path_content(path:&Path) -> StorageResult<Vec<PathBuf>> {
    Ok(try!(fs::read_dir(path))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>())
}


