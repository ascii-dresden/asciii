#![allow(dead_code, unused_variables)]
use std::fmt;
use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::{self, Write};
use std::str;

use git2;
use term::color;
use term::color::Color;

use manager::LuigiDir;

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
         GitStatus::WorkingNew      => color::GREEN,
         GitStatus::WorkingModified => color::YELLOW,
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
                statuses: statuses
            }
          )
    }

    fn cache_statuses(repo:&git2::Repository) -> Result<HashMap<PathBuf, GitStatus>, git2::Error>{
        use self::GitStatus::*;
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

    pub fn fetch(&self) -> Result<(), git2::Error>{
        let remote_name = "origin";

        // Figure out whether it's a named remote or a URL
        println!("Fetching {} for repo", remote_name);
        let mut cb = git2::RemoteCallbacks::new();
        let mut remote = try!(self.repo.find_remote(remote_name).or_else(|_| {
            self.repo.remote_anonymous(remote_name)
        }));
        cb.sideband_progress(|data| {
            print!("remote: {}", str::from_utf8(data).unwrap());
            io::stdout().flush().unwrap();
            true
        });

        cb.credentials(|str1, opt_str2, ty|{
            println!("{:?}; {:?}; {:?}", str1, opt_str2, ty.bits());
            //Err(git2::Error::from_str("test error"))
            git2::Cred::ssh_key_from_agent("hendrik")
        });

        // This callback gets called for each remote-tracking branch that gets
        // updated. The message we output depends on whether it's a new one or an
        // update.
        cb.update_tips(|refname, a, b| {
            if a.is_zero() {
                println!("[new]     {:20} {}", b, refname);
            } else {
                println!("[updated] {:10}..{:10} {}", a, b, refname);
            }
            true
        });

        // Here we show processed and total objects in the pack and the amount of
        // received data. Most frontends will probably want to show a percentage and
        // the download rate.
        cb.transfer_progress(|stats| {
            if stats.received_objects() == stats.total_objects() {
                print!("Resolving deltas {}/{}\r", stats.indexed_deltas(),
                stats.total_deltas());
            } else if stats.total_objects() > 0 {
                print!("Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_objects(),
                stats.received_bytes());
            }
            io::stdout().flush().unwrap();
            true
        });

        // Connect to the remote end specifying that we want to fetch information
        // from it.
        println!("canary 0");
        try!(remote.connect(git2::Direction::Fetch));
        println!("canary 1");

        // Download the packfile and index it. This function updates the amount of
        // received data and the indexer stats which lets you inform the user about
        // progress.
        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(cb);
        try!(remote.download(&[], Some(&mut fo)));

        println!("canary 2");
        {
            // If there are local objects (we got a thin pack), then tell the user
            // how many objects we saved from having to cross the network.
            let stats = remote.stats();
            if stats.local_objects() > 0 {
                println!("\rReceived {}/{} objects in {} bytes (used {} local \
                      objects)", stats.indexed_objects(),
                      stats.total_objects(), stats.received_bytes(),
                      stats.local_objects());
            } else {
                println!("\rReceived {}/{} objects in {} bytes",
                         stats.indexed_objects(), stats.total_objects(),
                         stats.received_bytes());
            }
        }
        println!("canary 3");

        // Disconnect the underlying connection to prevent from idling.
        remote.disconnect();

        // Update the references in the remote's namespace to point to the right
        // commits. This may be needed even if there was no packfile to download,
        // which can happen e.g. when the branches have been changed but all the
        // needed objects are available locally.
        try!(remote.update_tips(None, true,
                                git2::AutotagOption::Unspecified, None));

        Ok(())

    }
}
