//! contains `ProjectResult` and `ProjectError`, `ProductResult` and `ProductError`.
use std::io;
use std::fmt;
use std::error::Error;

pub type ProjectResult<T> = Result<T, ProjectError>;
pub type ProductResult<T> = Result<T, ProductError>;

#[derive(Debug)]
pub enum ProjectError {
    Io(io::Error),
    Fmt(fmt::Error),
    Product(ProductError),
    CantDetermineTargetFile,
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.cause() {
            None => write!(f, "{}", self.description(),),
            Some(cause) => write!(f, "{}", cause),
        }
    }
}

impl Error for ProjectError {
    fn description(&self) -> &str {
        match *self {
            ProjectError::CantDetermineTargetFile => "Cannot determine target file name",
            ProjectError::Io(ref inner) => inner.description(),
            ProjectError::Product(ref inner) => inner.description(),
            ProjectError::Fmt(ref inner) => inner.description(),
        }
    }
}

// All you need to make try!() fun again
impl From<io::Error> for ProjectError {
    fn from(io_error: io::Error) -> ProjectError {
        ProjectError::Io(io_error)
    }
}


// All you need to make try!() fun again
impl From<fmt::Error> for ProjectError {
    fn from(fmt_error: fmt::Error) -> ProjectError {
        ProjectError::Fmt(fmt_error)
    }
}


// All you need to make try!() fun again
impl From<ProductError> for ProjectError {
    fn from(error: ProductError) -> ProjectError {
        ProjectError::Product(error)
    }
}


#[derive(Debug, PartialEq, Eq)]
// #[derive(Debug)]
pub enum ProductError {
    // DuplicateProduct // not an error
    AmbiguousAmounts(String),
    MissingAmount(String),
    TooMuchReturned(String),
    InvalidPrice,
    UnknownFormat,
}

impl Error for ProductError{
    fn description(&self) -> &str {
        match *self {
            ProductError::AmbiguousAmounts(_) => "AmbiguousAmounts",
            ProductError::MissingAmount(_) => "Amounts missing",
            ProductError::TooMuchReturned(_) => "more returned than provided",
            ProductError::InvalidPrice => "invalid price",
            ProductError::UnknownFormat => "invalid format",
        }
    }
}

impl fmt::Display for ProductError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match self.cause() {
            None => write!(f, "{}", self.description(),),
            Some(cause) => write!(f, "{}", cause)
        }
    }
}

