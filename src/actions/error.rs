#![allow(trivial_casts)]


#[cfg(not(feature="document_export"))]
mod fake_error{
    //! this is not to be used, it only satisfies error_chain in case the `document_export` feature
    //! isn't used
    use std::error::Error;
    use std::fmt;
    #[derive(Debug)]
    pub struct RenderError;
    impl fmt::Display for RenderError{ fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self.cause() { None => write!(f, "{}", self.description(),), Some(cause) => write!(f, "{}", cause) } } }
    impl Error for RenderError{
        fn description(&self)->&str{"unimplemented"}
        fn cause(&self) -> Option<&Error>{None}
    }
}

#[cfg(feature="document_export")] use handlebars::RenderError;
#[cfg(not(feature="document_export"))] use self::fake_error::RenderError;

use std::io;
use std::fmt;
use std::time;
use std::error::Error as ErrorTrait;


use project;
//use project::error::ProjectError;
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
        project::error::Error, Project;
        StorageError, Storage;
    }

    errors {
        ActionError{
            description("unexpected response from service")
        }
    }
}
