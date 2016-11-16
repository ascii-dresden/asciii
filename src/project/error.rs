#![allow(trivial_casts)]

use std::io;
use std::fmt;
use util::yaml;

use super::product;

error_chain!{
    types {
        Error, ErrorKind, ChainErr, Result;
    }

    links {
        product::Error, product::ErrorKind, Product;
    }

    foreign_links {
        io::Error, Io;
        fmt::Error, Fmt;
        yaml::YamlError, Yaml;
    }

    errors {
        CantDetermineTargetFile{
            description("Cannot determine target file name")
        }
    }
}

pub type SpecResult = ::std::result::Result<(), ErrorList>;

pub struct ErrorList {
    pub errors: Vec<String>
}

impl ErrorList {
    pub fn new() -> Self{
        ErrorList {
            errors: Vec::new()
        }
    }

    pub fn push(&mut self, error:&str) {
        self.errors.push(error.into());
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
            try!(writeln!(f, " * {}", error))
        }
        Ok(())
    }
}
