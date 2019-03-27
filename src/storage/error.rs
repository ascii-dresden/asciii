#![allow(trivial_casts)]
//! Error that may occur in Storage
//!
#[cfg(feature = "git_statuses")]
use failure::Fail;

use std::path::PathBuf;
use std::{fmt, io};

use crate::util::yaml;
use crate::project::error::ProjectError;

#[cfg(not(feature = "git_statuses"))]
mod git2 {
    pub use super::super::repo::GitError as Error;
}


#[derive(Fail, Debug)]
pub enum StorageError {
    #[fail(display = "The directory you passed cannot be used in this context. You perhaps passed `Templates` instead of `Archive` or `Working`")]
    BadChoice,

    #[fail(display = "The Project file has a broken name.")]
    BadProjectFileName,

    #[fail(display = "There is no working directory.")]
    NoWorkingDir,

    #[fail(display = "Conflicting Name, you tried to create a project already exists.")]
    ProjectFileExists,

    #[fail(display = "Conflicting Name, you tried to create a project for which the project dir already exists.")]
    ProjectDirExists,

    #[fail(display = "No project was found matching this description.")]
    ProjectDoesNotExist,

    #[fail( display = "The project folder {:?} does not contain a project file.", _0)]
    NoProjectFile(PathBuf),

    #[fail(display = "Top Level storage path is not absolute.")]
    StoragePathNotAbsolute,

    #[fail(display = "The filestructure under storage path does not correspond with the configuration.")]
    InvalidDirStructure,

    #[fail(display = "The described template file does not exist.")]
    TemplateNotFound,

    #[fail(display = "Calling `git` failed")]
    GitProcessFailed,

    #[fail(display = "Git Repository was not initiliazed.")]
    RepoUnintialized,

    #[fail(display = "{:?}", _0)]
    String(String),

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    Fmt(#[cause] fmt::Error),

    #[fail(display = "{}", _0)]
    Yaml(#[cause] yaml::YamlError),

    #[fail(display = "{}", _0)]
    Project(#[cause] ProjectError),

    #[fail(display = "{}", _0)]
    Git(#[cause] git2::Error),
}

impl From<io::Error>  for StorageError { fn from(e: io::Error) ->  StorageError { StorageError::Io(e) } }
impl From<fmt::Error> for StorageError { fn from(e: fmt::Error) -> StorageError { StorageError::Fmt(e) } }
impl From<yaml::YamlError> for StorageError { fn from(e: yaml::YamlError) -> StorageError { StorageError::Yaml(e) } }
impl From<ProjectError> for StorageError { fn from(e: ProjectError) -> StorageError { StorageError::Project(e) } }

impl From<git2::Error> for StorageError { fn from(e: git2::Error) -> StorageError { StorageError::Git(e) } }

impl From<String> for StorageError { fn from(e: String) -> StorageError { StorageError::String(e) } }
impl From<&str> for StorageError { fn from(e: &str) -> StorageError { StorageError::String(e.into()) } }