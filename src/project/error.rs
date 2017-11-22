#![allow(trivial_casts)]

use std::{io, fmt};
#[cfg(feature="serialization")] use serde_json;
#[cfg(feature="deserialization")] use serde_yaml;
use util::yaml;

use super::product;

error_chain!{
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        Product(product::Error, product::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        Fmt(fmt::Error);
        Yaml(yaml::YamlError);
        Serialize(serde_json::Error) #[cfg(feature="serialization")];
        Deserialize(serde_yaml::Error) #[cfg(feature="deserialization")];
    }

    errors {
        FeatureDeactivated{
            description("This feature is not enabled in this build")
        }
        CantDetermineTargetFile{
            description("Cannot determine target file name")
        }
    }
}

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

#[derive(Default)]
pub struct ErrorList {
    pub errors: Vec<String>
}

impl ErrorList {
    pub fn new() -> Self{
        ErrorList {
            errors: Default::default()
        }
    }

    pub fn push(&mut self, error:&str) {
        self.errors.push(error.into());
    }

    pub fn combine_with(&self, other:&Self) -> Self{
        let mut new = ErrorList::new();
        for err in self.errors.iter().chain(other.errors.iter()) {
            new.push(err)
        }
        new
    }

    pub fn is_empty(&self) -> bool{
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, " * {}", error)?
        }
        Ok(())
    }
}
