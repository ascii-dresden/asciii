#![allow(missing_doc)]
use toml;
use std::{io, fmt, time};

use project;
// use project::error::ProjectError;
use storage::error::StorageError;

#[derive(Debug, Fail)]
pub enum Error {

    #[fail(display = "unexpected response from service")]
    ActionError,

    #[fail(display = "Adding Failed")]
    AddingFailed,

    #[fail(display = "{}", _0)]
    Io(io::Error),

    #[fail(display = "{}", _0)]
    Fmt(fmt::Error),

    #[fail(display = "{}", _0)]
    Time(time::SystemTimeError),

    #[fail(display = "{}", _0)]
    Toml(toml::de::Error),

    // TODO: Projecterror isn't Sync
    // #[fail(display = "{}", _0)]
    // Project(project::error::Error),

    // TODO: Projecterror isn't Send
    // #[fail(display = "{}", _0)]
    // Storage(StorageError),
}