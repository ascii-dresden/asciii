//! Implementations of `ProjectList`

use linked_hash_map::LinkedHashMap;
use std::iter::IntoIterator;
use std::ops::{Deref, DerefMut};

use super::{Storable, Year};

// TODO turn back on as soon as this is a feature
//pub type ProjectsByYear<P: Storable + Sized> = HashMap<Year, ProjectList<P>>;
pub type ProjectsByYear<P> = LinkedHashMap<Year, ProjectList<P>>;

#[derive(Debug)]
/// Container keeping all opened projects.
pub struct Projects<P: Storable + Sized> {
    /// working directory
    pub working: ProjectList<P>,
    /// archived Projects by year
    pub archive:  ProjectsByYear<P>
}


/// Wrapper around `Vec<Storable>`
///
/// This is produced by [`Storage::open_projects()`](struct.Storage.html#method.open_projects)
#[derive(Debug)]
pub struct ProjectList<P: Storable + Sized> {
    pub projects: Vec<P>
}

impl<L: Storable> ProjectList<L> {

    pub fn filter_by_all(&mut self, filters: &[&str]) {
        for filter in filters {
            self.filter_by(filter);
        }
    }

    pub fn filter_by_key_val(&mut self, key: &str, val: &str) {
        self.projects.retain(|p| p.matches_filter(key, val));
    }

    pub fn filter_by(&mut self, filter: &str) {
        let (key, val) = filter.split_at(filter.find(':').unwrap_or(0));
        self.filter_by_key_val(key, &val[1..]);
    }
}

impl<L: Storable> IntoIterator for ProjectList<L> {
    type Item = L;
    type IntoIter = ::std::vec::IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter{
        self.projects.into_iter()
    }
}

use std::iter::FromIterator;
impl<L: Storable> FromIterator<L> for ProjectList<L> {
    fn from_iter<I: IntoIterator<Item=L>>(iter: I) -> Self {
        let mut c = Vec::new();

        for i in iter {
            c.push(i);
        }

        ProjectList { projects: c }
    }
}

impl<L: Storable> Deref for ProjectList<L> {
    type Target = Vec<L>;
    fn deref(&self) -> &Vec<L> {
        &self.projects
    }
}

impl<L: Storable> DerefMut for ProjectList<L> {
    fn deref_mut(&mut self) -> &mut Vec<L> {
        &mut self.projects
    }
}


