//! Manages file structure of templates, working directory and archives.
//!
//! This module takes care of project file management.
//!
//! Your ordinary file structure would look something like this:
//!
//! ```bash
//! PROJECTS  # storage dir
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


#![allow(unused_imports)]

use std::path::{Path, PathBuf};
use std::marker::PhantomData;
use repo::Repository;

static PROJECT_FILE_EXTENSION:&'static str = "yml";
static TEMPLATE_FILE_EXTENSION:&'static str = "tyml";

/// Year = `i32`
pub type Year =  i32;

/// Result returned by Storage
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)] mod test;
#[cfg(test)] mod realworld;

mod project_list;
pub use self::project_list::ProjectList;
mod error;
pub mod traits;
pub use self::error::StorageError;
pub mod storable;
pub use self::storable::*;


#[cfg(feature = "new_storage")]
pub use self::traits::{Storing,Storage,GitStorage};


#[cfg(feature = "old_storage")]
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
/// * git interaction ( not yet )
#[cfg(feature = "old_storage")]
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

    pub repository: Option<Repository>
}

/// Used to identify what directory you are talking about.
#[derive(Debug,Clone)]
#[allow(dead_code)]
pub enum StorageDir { Working, Archive(Year), Root, Templates, All }

