#![allow(dead_code, unused_variables)]
use std::fmt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use git2;
use git2::*;
use git2::Error as GitError;
use term::color;
use term::color::Color;

use manager::LuigiDir;

#[derive(Debug,Clone)]
pub enum GitStatus{
    IndexNew, IndexModified , IndexDeleted, IndexRenamed, IndexTypechange,
    WorkingNew, WorkingModified, WorkingDeleted, WorkingTypechange, WorkingRenamed,
    Ignored, Conflict, Unknown
}

impl GitStatus{
    pub fn to_color(&self) -> Color {
        match *self{
        // => write!(f, "{:?}", self)
         GitStatus::Unknown         => color::YELLOW,
         GitStatus::Conflict        => color::RED,
         GitStatus::WorkingNew      => color::GREEN,
         GitStatus::WorkingModified => color::YELLOW,
         _                          => color::BLUE
        }
    }
}

impl fmt::Display for GitStatus{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match *self{
        // => write!(f, "{:?}", self)
         GitStatus::Unknown         => write!(f, " "),
         GitStatus::Conflict        => write!(f, "✘"),
         GitStatus::WorkingNew      => write!(f, "✚"),
         GitStatus::WorkingModified => write!(f, "~"),
         _                          => write!(f, "{:?}", self),

         //✘ ✔ ✚ ● ❌
        }
    }
}


pub struct Repo {
    pub repo: Repository,
    pub status: HashMap<PathBuf, GitStatus>
}

impl Repo{
    pub fn new(path:&Path) -> Result<Self, GitError> {
        let repo = try!(Repository::open(path));
        let status = try!(Self::status(&repo));
        Ok(Repo{
            repo: repo,
            status: status
        })
    }

    fn status(repo:&Repository) -> Result<HashMap<PathBuf, GitStatus>, GitError>{
        use self::GitStatus::*;
        let repo_path = repo.path().parent().unwrap().to_owned();

        let git_statuses = try!(repo.statuses( Some( StatusOptions::new()
                                                     .include_ignored(false)
                                                     .include_untracked(true) )));

        let mut statuses:HashMap<PathBuf,GitStatus> = HashMap::new();

        for entry in git_statuses.iter(){
            let status = match entry.status() {
                s if s.contains(STATUS_INDEX_NEW)        => IndexNew,
                s if s.contains(STATUS_INDEX_MODIFIED)   => IndexModified ,
                s if s.contains(STATUS_INDEX_DELETED)    => IndexDeleted,
                s if s.contains(STATUS_INDEX_RENAMED)    => IndexRenamed,
                s if s.contains(STATUS_INDEX_TYPECHANGE) => IndexTypechange,
                s if s.contains(STATUS_WT_NEW)           => WorkingNew,
                s if s.contains(STATUS_WT_MODIFIED)      => WorkingModified,
                s if s.contains(STATUS_WT_DELETED)       => WorkingDeleted,
                s if s.contains(STATUS_WT_TYPECHANGE)    => WorkingTypechange,
                s if s.contains(STATUS_WT_RENAMED)       => WorkingRenamed,
                s if s.contains(STATUS_IGNORED)          => Ignored,
                s if s.contains(STATUS_CONFLICTED)       => Conflict,
                _                                        => Unknown
            };

            if let Some(path) = entry.path(){
                let path = repo_path.join(PathBuf::from(path));
                if path.is_file() {
                    if let Some(parent) = path.parent(){
                        statuses.insert(parent.to_path_buf(), status.to_owned());
                    }
                }
                statuses.insert(path, status);
            }
        }

        Ok(statuses)
    }

    pub fn get_status(&self,path:&Path) -> GitStatus{
        self.status.get(path).unwrap_or(&GitStatus::Unknown).to_owned()
    }

    pub fn git_pull(&self) -> Result<(), GitError> {
        let remote = self.repo.find_remote("origin");
        unimplemented!();
    }

    pub fn git_add_directory(&self, name:&str, path:&Path) -> Result<(), GitError> {
        let index = self.repo.index();
        unimplemented!();
        //try!(index.add_path(&dir));
    }

    pub fn git_commit(&self, message:&str){
        unimplemented!();
    }

    pub fn git_push(&self){
        unimplemented!();
    }
}
