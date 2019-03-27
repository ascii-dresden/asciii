#![allow(trivial_casts)]


use failure::Fail;

use asciii::actions::error::ActionError;
use asciii::document_export::error::ExportError;
use asciii::project::error::ProjectError;
use asciii::storage::error::StorageError;

use std::{io, fmt, time};

pub type Result<T> = std::result::Result<T, CliError>;

#[cfg(feature="document_export")]

#[derive(Fail, Debug)]
pub enum CliError {
    #[fail(display = "This feature is not enabled in this build")]
    FeatureDeactivated,

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    Fmt(#[cause] fmt::Error),

    #[fail(display = "{}", _0)]
    Time(#[cause] time::SystemTimeError),

    #[fail(display = "{}", _0)]
    Action(#[cause] ActionError),

    #[fail(display = "{}", _0)]
    Export(#[cause] ExportError),

    #[fail(display = "{}", _0)]
    Project(#[cause] ProjectError),

    #[fail(display = "{}", _0)]
    Storage(#[cause] StorageError),

    #[fail(display = "{}", _0)]
    String(String),
}


impl From<io::Error>             for CliError { fn from(e: io::Error)            -> CliError { CliError::Io(e) } }
impl From<fmt::Error>            for CliError { fn from(e: fmt::Error)           -> CliError { CliError::Fmt(e) } }
impl From<ActionError>           for CliError { fn from(e: ActionError)          -> CliError { CliError::Action(e)} }
impl From<ExportError>           for CliError { fn from(e: ExportError)          -> CliError { CliError::Export(e)} }
impl From<ProjectError>          for CliError { fn from(e: ProjectError)         -> CliError { CliError::Project(e)} }
impl From<StorageError>          for CliError { fn from(e: StorageError)         -> CliError { CliError::Storage(e)} }
impl From<time::SystemTimeError> for CliError { fn from(e:time::SystemTimeError) -> CliError { CliError::Time(e) } }
 
impl From<String>                for CliError { fn from(e: String) -> CliError { CliError::String(e) } }
impl From<&str>                  for CliError { fn from(e: &str) -> CliError { CliError::String(e.into()) } }
 
