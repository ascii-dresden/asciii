#![allow(trivial_casts)]

use std::{io, fmt, time};
use std::error::Error as ErrorTrait;

use project;
// use project::error::ProjectError;
use storage::error::StorageError;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links { }

    foreign_links {
        Io(io::Error);
        Fmt(fmt::Error);
        Time(time::SystemTimeError);
        Project(project::error::Error);
        Storage(StorageError);
    }

    errors {
        ActionError{
            description("unexpected response from service")
        }
    }
}
