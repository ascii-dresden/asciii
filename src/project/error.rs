#![allow(trivial_casts)]
#![allow(missing_docs)]
#![allow(deprecated)]

use failure::Fail;

#[cfg(feature="serialization")] use serde_json;
#[cfg(feature="deserialization")] use serde_yaml;

use crate::util::yaml;
use crate::project::product::ProductError;
use super::spec::Validatable;

use std::{io, fmt};


#[derive(Fail, Debug)]
pub enum ProjectError {

    #[fail(display="This feature is not enabled in this build")]
    Product(ProductError),

    #[fail(display="This feature is not enabled in this build")]
    FeatureDeactivated,

    #[fail(display="Cannot determine target file name")]
    CantDetermineTargetFile,

    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    Fmt(#[cause] fmt::Error),

    #[fail(display = "{}", _0)]
    Yaml(#[cause] yaml::YamlError),

    #[cfg(feature="serialization")]
    #[fail(display = "{}", _0)]
    Serialize(serde_json::Error),

    #[cfg(feature="deserialization")]
    #[fail(display = "{}", _0)]
    Deserialize(serde_yaml::Error),
}


impl From<ProductError> for ProjectError { fn from(e: ProductError) -> ProjectError { ProjectError::Product(e) } }
impl From<io::Error> for ProjectError { fn from(e: io::Error) -> ProjectError { ProjectError::Io(e) } }
impl From<fmt::Error> for ProjectError { fn from(e: fmt::Error) -> ProjectError { ProjectError::Fmt(e) } }
impl From<yaml::YamlError> for ProjectError { fn from(e: yaml::YamlError) -> ProjectError { ProjectError::Yaml(e) } }
impl From<serde_json::Error> for ProjectError { fn from(e: serde_json::Error) -> ProjectError { ProjectError::Serialize(e) } }
impl From<serde_yaml::Error> for ProjectError { fn from(e: serde_yaml::Error) -> ProjectError { ProjectError::Deserialize(e) } }


pub type ProjectResult<T> = ::std::result::Result<T, failure::Error>;

pub type SpecResult = ::std::result::Result<(), ErrorList>;

pub fn combine_specresults(specs: Vec<SpecResult>) -> SpecResult {
    specs.into_iter()
         .fold(Ok(()), |acc, x|
                      match (acc, x) {
                          (Ok(_),          Ok(_))           => Ok(()),
                          (Err(left_list), Ok(_))           => Err(left_list),
                          (Ok(_),          Err(right_list)) => Err(right_list),
                          (Err(left_list), Err(right_list)) => Err(left_list.combine_with(&right_list))
                      }
                     )
}

pub fn all_check_out<T: Validatable>(specs: &[T]) -> SpecResult {
    let errors = specs.iter()
        .filter_map(Validatable::errors)
        .flat_map(|err_list| err_list.into_iter())
        .collect::<Vec<String>>();
    if !errors.is_empty() {
        Err(ErrorList::from_vec(errors))
    } else {
        Ok(())
    }
}

#[derive(Debug, Default, Fail)]
pub struct ErrorList {
    pub errors: Vec<String>
}

impl ErrorList {
    pub fn new() -> Self {
        ErrorList {
            errors: Default::default()
        }
    }

    pub fn from_vec(errors: Vec<String>) -> Self {
        ErrorList {
            errors
        }
    }

    pub fn push(&mut self, error: &str) {
        self.errors.push(error.into());
    }

    pub fn combine_with(&self, other: &Self) -> Self {
        let mut new = ErrorList::new();
        for err in self.errors.iter().chain(other.errors.iter()) {
            new.push(err)
        }
        new
    }

    pub fn combine_errors(lhs: &failure::Error, rhs: &failure::Error) -> Self {
        let left_list = lhs.downcast_ref::<Self>().unwrap();
        let right_list = rhs.downcast_ref::<Self>().unwrap();

        left_list.combine_with(&right_list)
    }

    pub fn into_iter(self) -> impl Iterator<Item=String> {
        self.errors.into_iter()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

use std::ops::Deref;
impl Deref for ErrorList {
    type Target = [String];
    fn deref(&self) -> &[String] {
        &self.errors
    }
}

impl<'a> From<&'a [&'a str]> for ErrorList {
    fn from(errs: &'a [&str]) -> ErrorList {
        let mut list = ErrorList::new();
        for e in errs {
            list.push(e);
        }
        list
    }
}


impl fmt::Display for ErrorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, " * {}", error)?
        }
        Ok(())
    }
}

