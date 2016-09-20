//! contains `ProjectResult` and `ProjectError`, `ProductResult` and `ProductError`.
use std::io;
use std::fmt;
use std::error::Error;

pub type ProjectResult<T> = Result<T, ProjectError>;
pub type ProductResult<T> = Result<T, ProductError>;

#[derive(Debug)]
pub enum ProjectError {
    Io(io::Error),
    CantDetermineTargetFile,
}

impl fmt::Display for ProjectError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self.cause() {
            None => write!(f, "{}", self.description(),),
            Some(cause) => write!(f, "{}", cause)
        }
    }
}

impl Error for ProjectError{
    fn description(&self) -> &str{
        match *self{
            ProjectError::CantDetermineTargetFile => "Cannot determine target file name",
            ProjectError::Io(ref inner)            => inner.description(),
        }
    }
}

// All you need to make try!() fun again
impl From<io::Error> for ProjectError {
    fn from(io_error: io::Error) -> ProjectError {
        ProjectError::Io(io_error)
    }
}


#[derive(Debug, PartialEq, Eq)]
//#[derive(Debug)]
pub enum ProductError {
    // DuplicateProduct // not an error
    AmbiguousAmounts(String),
    MissingAmount(String),
    TooMuchReturned(String),
    InvalidPrice,
    Unimplemented, // TODO remove
    UnknownFormat,
}
