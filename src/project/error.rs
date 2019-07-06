#![allow(missing_docs)]

use failure::Fail;
use std::iter::FromIterator;

use std::fmt;
use std::result::Result;

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

#[derive(Eq, PartialEq, Debug, Default, Fail)]
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

    pub fn into_vec(self) -> Vec<String> {
        self.errors
    }

    pub fn into_result(self) -> Result<(), Self> {

        if !self.is_empty() {
            return Err(self);
        }

        Ok(())
    }
}

use std::ops::Deref;
impl Deref for ErrorList {
    type Target = [String];
    fn deref(&self) -> &[String] {
        &self.errors
    }
}

impl FromIterator<String> for ErrorList {
    fn from_iter<I: IntoIterator<Item=String>>(iter: I) -> Self {
        let mut list = ErrorList::new();
        for e in iter {
            list.errors.push(e);
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

