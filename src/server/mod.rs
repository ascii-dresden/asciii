#![cfg(feature = "server")]

use linked_hash_map::LinkedHashMap;
use itertools::Itertools;

use ::project::Project;
use ::storage::{self, ProjectList, Storage, StorageDir, Storable};

pub struct ProjectLoader {
    pub storage: Storage<Project>,
    pub state: State,
}



pub struct State {
    pub all: ProjectList<Project>,
    pub working: LinkedHashMap<String, Project>,
    pub mapped:  LinkedHashMap<String, Project>,
    pub years: Vec<i32>
}

fn reinitialize(storage: &Storage<Project>) -> State {
    let all = storage.open_projects(StorageDir::All).unwrap();

    let working = storage.open_projects(StorageDir::Working).unwrap()
        .into_iter()
        .map(|p| (format!("{}", Storable::ident(&p)), p))
        .collect();

    let mapped = all.iter()
        .cloned()
        .map(|p| (format!("{}-{}",
                          Storable::year(&p).unwrap(),
                          Storable::ident(&p)),
                          p))
        .collect();

    let years = all.iter()
                   .filter_map(|p: &Project| p.year())
                   .unique()
                   .collect::<Vec<_>>();

    State {all, working, mapped, years}
}

impl<'a> ProjectLoader {

    pub fn new() -> Self {
        let storage = storage::setup().unwrap();
        let state = reinitialize(&storage);

        Self {
            storage,
            state
        }
    }

    pub fn update(&mut self) {
        debug!("updating projects");
        self.state = reinitialize(&self.storage);
    }
}
