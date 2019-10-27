//! Error that may occur in Storage
//!
use failure::Fail;

use std::path::PathBuf;

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

    #[fail(display = "Faulty config: {} does not contain the expected value", _0)]
    FaultyConfig(String),

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

    #[fail(display = "The file structure under storage path does not correspond with the configuration.")]
    InvalidDirStructure,

    #[fail(display = "The described template file does not exist.")]
    TemplateNotFound,

    #[fail(display = "Calling `git` failed")]
    GitProcessFailed,

    #[fail(display = "Git Repository was not initialized.")]
    RepoUninitialized,

    #[fail(display = "Nothing found for {:?}", _0)]
    NothingFound(Vec<String>),

}
