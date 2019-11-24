#![allow(missing_doc)]
use toml;
use std::{io, fmt, time};

use project;
// use project::error::ProjectError;
use storage::error::StorageError;

#[derive(Debug, Error)]
pub enum Error {

    #[error("unexpected response from service")]
    ActionError,

    #[error("Adding Failed")]
    AddingFailed,

    #[error("{}", _0)]
    Io(io::Error),

    #[error("{}", _0)]
    Fmt(fmt::Error),

    #[error("{}", _0)]
    Time(time::SystemTimeError),

    #[error("{}", _0)]
    Toml(toml::de::Error),

    // TODO: Projecterror isn't Sync
    // #[error("{}", _0)]
    // Project(project::error::Error),

    // TODO: Projecterror isn't Send
    // #[error("{}", _0)]
    // Storage(StorageError),
}