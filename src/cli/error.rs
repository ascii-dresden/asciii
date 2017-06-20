#![allow(trivial_casts)]

use std::io;
use std::fmt;
use std::time;

use asciii::actions;
use asciii::project;
use asciii::storage;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links { }

    foreign_links {
        Io(io::Error);
        Fmt(fmt::Error);
        Time(time::SystemTimeError);
        Actions(actions::error::Error);
        Project(project::error::Error);
        Storage(storage::error::StorageError);
    }

    errors {
    }
}
