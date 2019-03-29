#![allow(missing_docs)]

use failure::Fail;

use std::fmt;

#[derive(Fail, Debug)]
pub enum ProjectError {

    #[fail(display="This feature is not enabled in this build")]
    FeatureDeactivated,

    #[fail(display="Cannot determine target file name")]
    CantDetermineTargetFile,
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

