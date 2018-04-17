use itertools::Itertools;
use linked_hash_map::LinkedHashMap;

use asciii::project::Project;
use asciii::storage::{self, ProjectList, Storable, Storage, StorageDir};

use std::convert::TryInto;

pub mod endpoints;

pub struct ProjectLoader {
    pub(crate) storage:      Storage<Project>,
    pub(crate) years:        Vec<i32>,
    pub(crate) projects_all: ProjectList<Project>,
    pub(crate) projects_map: LinkedHashMap<String, Project>,
}

impl<'a> ProjectLoader {
    pub fn new() -> Self {
        let storage = storage::setup().unwrap();
        let projects_all = storage.open_projects(StorageDir::All).unwrap();
        let projects_map = storage
            .open_projects(StorageDir::All)
            .unwrap()
            .into_iter()
            .map(|p| (format!("{}-{}", Storable::year(&p).unwrap(), Storable::ident(&p)), p))
            .collect();
        let years = projects_all
            .iter()
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

#[derive(FromForm, Debug)]
pub struct Dir {
    pub year: Option<i32>,
    pub all:  Option<bool>,
}

impl TryInto<StorageDir> for Dir {
    type Error = String;

    fn try_into(self) -> Result<StorageDir, Self::Error> {
        let dir = match self {
            Dir{ all: Some(true), year: None } => StorageDir::All,
            Dir{ all: Some(true), year: Some(_) } => return Err("Ambiguous".into()),
            Dir{ all: None, year: Some(year) } => StorageDir::Archive(year),
            Dir{ all: None, year: None } => StorageDir::Working,
            _ => StorageDir::Working,
        };
        Ok(dir)
    }
}

