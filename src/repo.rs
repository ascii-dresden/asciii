#![allow(dead_code, unused_variables)]
use std::fmt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::{self, Write};
use std::str;

use git2;
use git2::Error as GitError;
use git2::*;
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


pub trait EasyGit{
    /// takes an absolute file and returns a usable status
    fn status_for_path(&self, path:&Path) -> Option<GitStatus>;

}

impl EasyGit for Repository{
    fn status_for_path(&self, path:&Path) -> Option<GitStatus>{
        self.path().parent()
            .and_then(|root| path.relative_from(root))
            .and_then(|rel_path| self.status_file(&rel_path).ok())
            .map(|status_bits|bits_to_status(&status_bits))
    }
}

fn bits_to_status(status_bits:&git2::Status) -> GitStatus{
    match status_bits {
        s if s.contains(STATUS_INDEX_NEW)        => GitStatus::IndexNew,
        s if s.contains(STATUS_INDEX_MODIFIED)   => GitStatus::IndexModified ,
        s if s.contains(STATUS_INDEX_DELETED)    => GitStatus::IndexDeleted,
        s if s.contains(STATUS_INDEX_RENAMED)    => GitStatus::IndexRenamed,
        s if s.contains(STATUS_INDEX_TYPECHANGE) => GitStatus::IndexTypechange,
        s if s.contains(STATUS_WT_NEW)           => GitStatus::WorkingNew,
        s if s.contains(STATUS_WT_MODIFIED)      => GitStatus::WorkingModified,
        s if s.contains(STATUS_WT_DELETED)       => GitStatus::WorkingDeleted,
        s if s.contains(STATUS_WT_TYPECHANGE)    => GitStatus::WorkingTypechange,
        s if s.contains(STATUS_WT_RENAMED)       => GitStatus::WorkingRenamed,
        s if s.contains(STATUS_IGNORED)          => GitStatus::Ignored,
        s if s.contains(STATUS_CONFLICTED)       => GitStatus::Conflict,
        _                                        => GitStatus::Unknown
    }
}

pub fn check_statuses(repo:&Repository) -> Result<HashMap<PathBuf, GitStatus>, GitError>{
    use self::GitStatus::*;
    let repo_path = repo.path().parent().unwrap().to_owned();

    let git_statuses = try!(repo.statuses( Some( StatusOptions::new()
                                                 .include_ignored(false)
                                                 .include_untracked(true) )));

    let mut statuses:HashMap<PathBuf,GitStatus> = HashMap::new();

    for entry in git_statuses.iter(){
        let status = bits_to_status(&entry.status());

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

