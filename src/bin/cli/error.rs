#![allow(trivial_casts)]

use std::io;
use std::fmt;
use std::time;

use asciii::actions;
use asciii::project;
use asciii::storage;
#[cfg(feature="document_export")]
use asciii::document_export;

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
        Export(document_export::error::Error) #[cfg(feature="document_export")];
        Project(project::error::Error);
        Storage(storage::error::StorageError);
    }

    errors {
        FeatureDeactivated{
            description("This feature is not enabled in this build")
        }
    }
}
