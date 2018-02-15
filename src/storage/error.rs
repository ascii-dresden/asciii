//! Error that may occur in Storage
//!

#![allow(trivial_casts)]
use std::{io, fmt};
use ::project;
#[cfg(feature="git_statuses")] use git2;

#[cfg(not(feature="git_statuses"))]
mod git2 {
    pub use super::super::repo::GitError as Error;
}

use templater;

#[allow(missing_docs)]
error_chain!{
    types {
        StorageError, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Io(io::Error);
        Fmt(fmt::Error);
        Git(git2::Error);
        Project(project::error::Error); // TODO this should be generic
        Template(templater::TemplateError); // this should also not be here (inversion)
    }

    errors {
        BadChoice {
            description( "The directory you passed cannot be used in this context. You perhaps passed `Templates` instead of `Archive` or `Working`")}
        BadProjectFileName {
            description("The Project file has a broken name.")
        }
        NoWorkingDir {
            description("There is no working directory.")
        }
        ProjectFileExists {
            description("Conflicting Name, you tried to create a project already exists.")
        }
        ProjectDirExists {
            description("Conflicting Name, you tried to create a project for which the project dir already exists.")
        }
        ProjectDoesNotExist {
            description("No project was found matching this description.")
        }
        StoragePathNotAbsolute {
            description("Top Level storage path is not absolute.")
        }
        InvalidDirStructure {
            description("The filestructure under storage path does not correspond with the configuration.")
        }
        TemplateNotFound {
            description("The described template file does not exist.")
        }
        GitProcessFailed {
            description("Calling `git` failed")
        }
    }
}
