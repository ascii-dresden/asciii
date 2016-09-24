#![allow(trivial_casts)]

use handlebars::RenderError;

use std::io;
use std::fmt;
use std::time;
use std::error::Error as ErrorTrait;

use project::error::ProjectError;
use storage::error::StorageError;

error_chain!{

    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links {
    }

    foreign_links {
        io::Error, Io;
        fmt::Error, Fmt;
        time::SystemTimeError, Time;
        RenderError, Handlebar;
        ProjectError, Project;
        StorageError, Storage;
    }

    errors {
        ActionError{
            description("unexpected response from service")
        }
    }
}
