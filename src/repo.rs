#![allow(dead_code, unused_variables)]
use std::fmt;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::{self, Write};
use std::str;

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

    pub fn fetch(&self) -> Result<(), GitError> {
        let repo = &self.repo;
        let remote = "origin";

        // Figure out whether it's a named remote or a URL
        println!("Fetcing {} for repo", remote);
        let mut cb = RemoteCallbacks::new();
        let mut remote = try!(repo.find_remote(remote).or_else(|_| {
            repo.remote_anonymous(remote)
        }));
        cb.sideband_progress(|data| {
            print!("remote: {}", str::from_utf8(data).unwrap());
            io::stdout().flush().unwrap();
            true
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
        try!(remote.connect(Direction::Fetch));

        // Download the packfile and index it. This function updates the amount of
        // received data and the indexer stats which lets you inform the user about
        // progress.
        let mut fo = FetchOptions::new();
        fo.remote_callbacks(cb);
        try!(remote.download(&[], Some(&mut fo)));

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

        // Disconnect the underlying connection to prevent from idling.
        remote.disconnect();

        // Update the references in the remote's namespace to point to the right
        // commits. This may be needed even if there was no packfile to download,
        // which can happen e.g. when the branches have been changed but all the
        // needed objects are available locally.
        try!(remote.update_tips(None, true,
                                AutotagOption::Unspecified, None));

        Ok(())
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
