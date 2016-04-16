//! Manages file structure of templates, working directory and archive.
//!
//! This module takes care of all the plumbing underneath,
//! that is why it's called *"Storage"*.
//!
//! Your ordinary file structure would look something like this:
//!
//! ```bash
//! PROJECTS  # storage dir
//! ├── working
//! │   └── Project1
//! │       └── Project1.yml
//! ├── archive
//! │   ├── 2013
//! │   └── 2014
//! │       └── R036_Project3
//! │           ├── Project3.yml
//! │           └── R036 Project3 2014-10-08.tex
//! ...
//! ```
//!

#![allow(dead_code)]

use std::ops::{Deref, DerefMut};


static PROJECT_FILE_EXTENSION:&'static str = "yml";
static TEMPLATE_FILE_EXTENSION:&'static str = "tyml";

pub type Year =  i32;
pub type StorageResult<T> = Result<T, StorageError>;

#[cfg(test)]
mod test ;

#[cfg(test)]
mod realworld;

mod error;
pub use self::error::StorageError;

pub mod storage;
pub use self::storage::Storage;

pub mod storable;
pub use self::storable::Storable;

//pub mod storable;


/// Used to identify what directory you are talking about.
#[derive(Debug,Clone)]
pub enum StorageDir { Working, Archive(Year), Storage, Templates, All }

//TODO implement Display for StorageError or use Quickerror



/// Wrapper around `Vec<Storable>`
pub struct ProjectList<P:Storable+Sized>{
    pub projects: Vec<P>
}

impl<L:Storable> ProjectList<L>{

    pub fn filter_by_all(&mut self, filters:&[&str]){
        for filter in filters{
            self.filter_by(filter);
        }
    }

    pub fn filter_by(&mut self, filter:&str){
        self.projects.retain(|p|{
            let (key,val) = filter.split_at(filter.find(':').unwrap_or(0));
            p.matches_filter(&key, &val[1..])
        });
    }

}

impl<L:Storable> Deref for ProjectList<L>{
    type Target=Vec<L>;
    fn deref(&self) -> &Vec<L>{
        &self.projects
    }
}

impl<L:Storable> DerefMut for ProjectList<L>{
    fn deref_mut(&mut self) -> &mut Vec<L>{
        &mut self.projects
    }
}


