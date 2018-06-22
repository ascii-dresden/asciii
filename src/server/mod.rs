#![cfg(feature = "server")]

use linked_hash_map::LinkedHashMap;
use itertools::Itertools;

use ::project::Project;
use ::storage::{self, ProjectList, Storage, StorageDir, Storable};

pub struct ProjectLoader {
    pub storage: Storage<Project>,
    pub years: Vec<i32>,
    pub projects_all: ProjectList<Project>,
    pub projects_map: LinkedHashMap<String, Project>,
}

impl<'a> ProjectLoader {
    pub fn new() -> Self {

        let storage = storage::setup().unwrap();
        let projects_all = storage.open_projects(StorageDir::All).unwrap();
        let projects_map = storage.open_projects(StorageDir::All)
            .unwrap()
            .into_iter()
            .map(|p| (format!("{}-{}",
                              Storable::year(&p).unwrap(),
                              Storable::ident(&p)),
                              p))
            .collect();
        let years = projects_all.iter()
                                    .filter_map(|p: &Project| p.year())
                                    .unique()
                                    .collect::<Vec<_>>();

        Self {
            storage,
            years,
            projects_all,
            projects_map,
        }
    }

    pub fn update(&mut self) {
        debug!("updating projects");
        self.projects_all = self.storage.open_projects(StorageDir::All).unwrap();
    }
}
