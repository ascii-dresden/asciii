//! Implementations of `ProjectList`

use std::ops::{Deref, DerefMut};

use super::Storable;

/// Wrapper around `Vec<Storable>`
pub struct ProjectList<P: Storable + Sized> {
    pub projects: Vec<P>,
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

use std::iter::IntoIterator;
impl<L: Storable> IntoIterator for ProjectList<L> {
    type Item = L;
    type IntoIter = ::std::vec::IntoIter<L>;

    fn into_iter(self) -> Self::IntoIter{
        self.projects.into_iter()
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
