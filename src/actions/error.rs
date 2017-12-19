#![allow(trivial_casts)]

use toml;
use std::{io, fmt, time};

use project;
// use project::error::ProjectError;
use storage::error::StorageError;

#[allow(missing_doc)]
error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links { }

    foreign_links {
        Io(io::Error);
        Fmt(fmt::Error);
        Time(time::SystemTimeError);
        Toml(toml::de::Error);
        Project(project::error::Error);
        Storage(StorageError);
    }

    errors {
        ActionError{
            description("unexpected response from service")
        }
        AddingFailed{
            description("Adding Failed")
        }
    }
}
