#![allow(dead_code, unused_variables)]
use std::fmt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command,ExitStatus};

use git2;
use term::color;
use term::color::Color;

/// More Rustacious way of representing a git status
#[derive(Debug,Clone)]
pub enum GitStatus{
    IndexNew, IndexModified , IndexDeleted, IndexRenamed, IndexTypechange,
    WorkingNew, WorkingModified, WorkingDeleted, WorkingTypechange, WorkingRenamed,
    Ignored, Conflict, Current, Unknown
}

impl GitStatus{
    pub fn to_color(&self) -> Color {
        match *self{
        // => write!(f, "{:?}", self)
         GitStatus::Current         => color::BLUE,
         GitStatus::Conflict        => color::RED,
         GitStatus::WorkingNew      => color::YELLOW,
         GitStatus::WorkingModified => color::YELLOW,
         GitStatus::IndexNew        => color::GREEN,
         //GitStatus::Unknown         => color::WHITE,
         _                          => color::WHITE
        }
    }
}

impl fmt::Display for GitStatus{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        match *self{
        // => write!(f, "{:?}", self)
         GitStatus::Conflict        => write!(f, "X"),
         GitStatus::Current         => write!(f, "+"),
         GitStatus::WorkingNew      => write!(f, "+"),
         GitStatus::WorkingModified => write!(f, "~"),
         GitStatus::IndexNew        => write!(f, "+"),
         GitStatus::Unknown         => write!(f, ""),
         _                          => write!(f, "{:?}", self),

        }
    }
}

impl From<git2::Status> for GitStatus{
    fn from(status:git2::Status) -> Self{
        match status{
            //s if s.contains(git2::STATUS_CURRENT)          => GitStatus::Current,
            s if s.contains(git2::STATUS_INDEX_NEW)        => GitStatus::IndexNew,
            s if s.contains(git2::STATUS_INDEX_MODIFIED)   => GitStatus::IndexModified ,
            s if s.contains(git2::STATUS_INDEX_DELETED)    => GitStatus::IndexDeleted,
            s if s.contains(git2::STATUS_INDEX_RENAMED)    => GitStatus::IndexRenamed,
            s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => GitStatus::IndexTypechange,
            s if s.contains(git2::STATUS_WT_NEW)           => GitStatus::WorkingNew,
            s if s.contains(git2::STATUS_WT_MODIFIED)      => GitStatus::WorkingModified,
            s if s.contains(git2::STATUS_WT_DELETED)       => GitStatus::WorkingDeleted,
            s if s.contains(git2::STATUS_WT_TYPECHANGE)    => GitStatus::WorkingTypechange,
            s if s.contains(git2::STATUS_WT_RENAMED)       => GitStatus::WorkingRenamed,
            s if s.contains(git2::STATUS_IGNORED)          => GitStatus::Ignored,
            s if s.contains(git2::STATUS_CONFLICTED)       => GitStatus::Conflict,
            _                                              => GitStatus::Unknown
        }
    }
}

/// Convenience Wrapper for `git2::Repository`
pub struct Repository{
    /// Git Repository for StorageDir
    pub repo: git2::Repository,
    pub workdir: PathBuf,
    /// Maps GitStatus to each path
    pub statuses: HashMap<PathBuf, GitStatus>
}

impl Repository {
    pub fn new(path:&Path) -> Result<Self, git2::Error>{
        let repo = try!(git2::Repository::open(path));
        let statuses = try!(Self::cache_statuses(&repo));
        Ok(
            Repository{
                repo: repo,
                workdir: path.to_owned(),
                statuses: statuses
            }
          )
    }

    fn cache_statuses(repo:&git2::Repository) -> Result<HashMap<PathBuf, GitStatus>, git2::Error>{
        let repo_path = repo.path().parent().unwrap().to_owned();

        let git_statuses = try!(repo.statuses( Some( git2::StatusOptions::new()
                                                     .include_ignored(false)
                                                     .include_untracked(true) )));

        let mut statuses:HashMap<PathBuf,GitStatus> = HashMap::new();

        for entry in git_statuses.iter(){
            let status:GitStatus = entry.status().into();

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

    /// Returns the status to a given path
    pub fn get_status(&self,path:&Path) -> GitStatus{
        self.statuses.get(path).unwrap_or(&GitStatus::Unknown).to_owned()
    }

    fn execute_git(&self, command:&str, args:&[&str]) -> ExitStatus{
        let workdir = self.repo.workdir().unwrap();
        let gitdir  = workdir.join(".git");


        debugln!("{:#?}", Command::new("git")
                 .args(&["--work-tree", workdir.to_str().unwrap()])
                 .args(&["--git-dir",   gitdir.to_str().unwrap()])
                 .arg(command).args(args));

        Command::new("git")
            .args(&["--work-tree", workdir.to_str().unwrap()])
            .args(&["--git-dir",   gitdir.to_str().unwrap()])
            .arg(command)
            .args(args)
            .status()
            .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) })
    }

    pub fn add(&self, paths:&[PathBuf]) -> ExitStatus{
        println!("adding to git {:?}", paths);
        let paths:Vec<&str> = paths.iter().filter_map(|p|p.to_str()).collect();
        self.execute_git("add", &paths)
    }

    pub fn commit(&self) -> ExitStatus{
        // TODO override git editor with asciii editor
        self.execute_git("commit", &[])
    }

    pub fn status(&self) -> ExitStatus{
        self.execute_git("status", &["origin", "master"])
    }

    pub fn push(&self) -> ExitStatus{
        self.execute_git("push", &["origin", "master"])
    }

    pub fn pull(&self) -> ExitStatus{
        self.execute_git("pull", &["origin", "master"])
    }
}
