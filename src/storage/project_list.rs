//! Implementations of `ProjectList`

use std::ops::{Deref, DerefMut};

use super::Storable;
use super::ProjectList;

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


