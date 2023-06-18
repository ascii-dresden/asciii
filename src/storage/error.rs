//! Error that may occur in Storage
//!
use std::path::PathBuf;

#[cfg(not(feature = "git_statuses"))]
mod git2 {
    pub use super::super::repo::GitError as Error;
}

#[derive(thiserror::Error, Debug)]
pub enum StorageError {
    #[error("The directory you passed cannot be used in this context. You perhaps passed `Templates` instead of `Archive` or `Working`")]
    BadChoice,

    #[error("The Project file has a broken name.")]
    BadProjectFileName,

    #[error("There is no working directory.")]
    NoWorkingDir,

    #[error("Faulty config: {} does not contain the expected value", _0)]
    FaultyConfig(String),

    #[error("Conflicting Name, you tried to create a project already exists.")]
    ProjectFileExists,

    #[error("Conflicting Name, you tried to create a project for which the project dir already exists.")]
    ProjectDirExists,

    #[error("No project was found matching this description.")]
    ProjectDoesNotExist,

    #[error("The project folder {:?} does not contain a project file.", _0)]
    NoProjectFile(PathBuf),

    #[error("Top Level storage path is not absolute.")]
    StoragePathNotAbsolute,

    #[error("The file structure under storage path does not correspond with the configuration.")]
    InvalidDirStructure,

    #[error("The described template file does not exist.")]
    TemplateNotFound,

    #[error("Calling `git` failed")]
    GitProcessFailed,

    #[error("Git Repository was not initialized.")]
    RepoUninitialized,

    #[error("Nothing found for {:?}", _0)]
    NothingFound(Vec<String>)
}
