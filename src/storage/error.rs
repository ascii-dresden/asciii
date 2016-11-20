//! Error that may occur in Storage
//!
#![allow(trivial_casts)]
use std::io;
use std::fmt;
use git2;
use util::yaml;

use templater;

#[cfg(not(feature="git_statuses"))]
use super::repo::GitError;

error_chain!{
    types {
        StorageError, ErrorKind, Result;
    }

    foreign_links {
        io::Error, Io;
        fmt::Error, Fmt;
        yaml::YamlError, Yaml;
        git2::Error, Git;
        templater::TemplateError, Template;
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
