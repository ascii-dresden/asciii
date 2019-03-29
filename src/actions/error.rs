use toml;
use failure::Fail;
use std::{io, fmt, time};

use crate::project::error::ProjectError;
use crate::storage::error::StorageError;

#[derive(Fail, Debug)]
pub enum ActionError {

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    Fmt(#[cause] fmt::Error),

    #[fail(display = "{}", _0)]
    Time(#[cause] time::SystemTimeError),

    #[fail(display = "{}", _0)]
    Toml(#[cause] toml::de::Error),

    #[fail(display = "unexpected response from service")]
    ActionError,

    #[fail(display = "Adding Failed")]
    AddingFailed,

    #[fail(display = "Nothing found for {:?}", _0)]
    NothingFound(Vec<String>),

    #[fail(display = "{}", _0)]
    Project(#[cause] ProjectError),

    #[fail(display = "{}", _0)]
    Storage(#[cause] StorageError),
}

impl From<io::Error>   for ActionError { fn from(e: io::Error) ->  ActionError { ActionError::Io(e) } }
impl From<fmt::Error>  for ActionError { fn from(e: fmt::Error) -> ActionError { ActionError::Fmt(e) } }
impl From<time::SystemTimeError> for ActionError { fn from(e:time::SystemTimeError) -> ActionError { ActionError::Time(e) } }
impl From<toml::de::Error> for ActionError { fn from(e: toml::de::Error) -> ActionError { ActionError::Toml(e) } }

impl From <ProjectError> for ActionError {fn from(e: ProjectError) -> ActionError { ActionError::Project(e)} }
impl From <StorageError> for ActionError {fn from(e: StorageError) -> ActionError { ActionError::Storage(e)} }