//! Error that may occur in Storage
//!
//TODO implement Display for StorageError or use Quickerror
use std::io;
use std::fmt;
use std::error::Error;


#[cfg(feature="git_statuses")]
use git2::Error as GitError;

use templater::TemplateError;
use util::yaml::YamlError;

#[cfg(not(feature="git_statuses"))]
use super::repo::GitError;

/// Error that may occur in Storage
#[derive(Debug)]
pub enum StorageError {
    BadChoice, // TODO: this should be a compile error
    BadProjectFileName,
    NoWorkingDir,
    ProjectFileExists,
    ProjectDirExists,
    ProjectDoesNotExist,
    StoragePathNotAbsolute,
    InvalidDirStructure,
    ParseError(YamlError), // TODO: Make ParseError more generic
    TemplateNotFound,
    Git(GitError),
    Io(io::Error),
    Template(TemplateError),
}

impl fmt::Display for StorageError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self.cause() {
            None => write!(f, "{}", self.description(),),
            Some(cause) => write!(f, "{}", cause)
        }
    }
}

impl Error for StorageError{
    fn description(&self) -> &str{
        match *self{
            StorageError::BadChoice                => "The directory you passed cannot be used in this context. You perhaps passed `Templates` instead of `Archive` or `Working`",
            StorageError::BadProjectFileName       => "The Project file has a broken name.",
            StorageError::NoWorkingDir             => "There is no working directory.",
            StorageError::ProjectFileExists        => "Conflicting Name, you tried to create a project already exists.",
            StorageError::ProjectDirExists         => "Conflicting Name, you tried to create a project for which the project dir already exists.",
            StorageError::ProjectDoesNotExist      => "No project was found matching this description.",
            StorageError::StoragePathNotAbsolute   => "Top Level storage path is not absolute.",
            StorageError::InvalidDirStructure      => "The filestructure under storage path does not correspond with the configuration.",
            StorageError::ParseError(ref inner)    => inner.description(),
            StorageError::TemplateNotFound         => "The described template file does not exist.",
            StorageError::Git(ref inner)           => inner.description(),
            StorageError::Io(ref inner)            => inner.description(),
            StorageError::Template(ref inner)      => inner.description(),
        }
    }

    fn cause(&self) -> Option<&Error>{
        match *self{
            StorageError::ParseError(ref inner)          => Some(inner),
            StorageError::Git(ref inner)                 => Some(inner),
            StorageError::Io(ref inner)                  => Some(inner),
            StorageError::Template(ref inner)            => Some(inner),
            _                                            => None
        }
    }
}

// All you need to make try!() fun again
impl From<io::Error>  for StorageError {
    fn from(io_error: io::Error) -> StorageError{ StorageError::Io(io_error) }
}

impl From<GitError>  for StorageError {
    fn from(git_error: GitError) -> StorageError{ StorageError::Git(git_error) }
}

impl From<TemplateError>  for StorageError {
    fn from(template_error: TemplateError) -> StorageError{
        StorageError::Template(template_error)
    }
}

