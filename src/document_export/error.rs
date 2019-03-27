use failure::Fail;
use handlebars::RenderError;

use crate::project::error::ProjectError;
use crate::storage::error::StorageError;

use std::{io, fmt, path::PathBuf, time};

pub type ExportResult<T> = Result<T, ExportError>;

#[derive(Fail, Debug)]
pub enum ExportError {
    #[fail(display = "No PDF Created")]
    NoPdfCreated,

    #[fail(display = "Nothing to do")]
    NothingToDo,

    #[fail(display = "Template not found at {:?}", _0)]
    TemplateNotFoundAt(PathBuf),

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    Fmt(#[cause] fmt::Error),

    #[fail(display = "{}", _0)]
    Time(#[cause] time::SystemTimeError),

    #[fail(display = "{}", _0)]
    Handlebar(#[cause] RenderError),

    #[fail(display = "{}", _0)]
    Project(#[cause] ProjectError),

    #[fail(display = "{}", _0)]
    Storage(#[cause] StorageError),
}


impl From<io::Error> for ExportError { fn from(e: io::Error) -> ExportError { ExportError::Io(e) } }
impl From<fmt::Error> for ExportError { fn from(e: fmt::Error) -> ExportError { ExportError::Fmt(e) } }
impl From<ProjectError> for ExportError {fn from(e: ProjectError) -> ExportError { ExportError::Project(e)} }
impl From<StorageError> for ExportError {fn from(e: StorageError) -> ExportError { ExportError::Storage(e)} }
impl From<RenderError> for ExportError {fn from(e: RenderError) -> ExportError { ExportError::Handlebar(e)} }
impl From<time::SystemTimeError> for ExportError { fn from(e:time::SystemTimeError) -> ExportError { ExportError::Time(e) } }
 
 
