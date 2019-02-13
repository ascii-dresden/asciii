use clap::ArgMatches;
use log::{trace, error};

use asciii::{storage, util};
use asciii::project::Project;

use super::matches_to_paths;
use crate::cli::error::*;

/// Command LOG
pub fn git_log(matches: &ArgMatches<'_>) -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let paths = matches_to_paths(matches, &storage)?;
    let repo = storage.repository().unwrap();
    if !repo.log(&paths).success() {
        Err("git log did not exit successfully".into())
    } else {
        Ok(())
    }
}

/// Command STATUS
pub fn git_status() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.status().success() {
        Err("git status did not exit successfully".into())
    } else {
        Ok(())
    }
}

/// Command COMMIT
pub fn git_commit() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.commit().success() {
        Err("git commit did not exit successfully".into())
    } else {
        Ok(())
    }
}

/// Get git `user.name`
///
/// `git config --local user.name`

/// Command REMOTE
/// exact replica of `git remote -v`
#[cfg(not(feature = "git_statuses"))]
pub fn git_remote() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    storage.repository().unwrap().remote();
    Ok(())
}

/// Command REMOTE
/// exact replica of `git remote -v`
#[cfg(feature = "git_statuses")]
pub fn git_remote() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;

    if let Some(r) = storage.repository() {
        let repo = &r.repo;

        for remote_name in repo.remotes().unwrap().iter() {

            if let Some(name) = remote_name {

                if let Ok(remote) = repo.find_remote(name) {
                    println!("{}", lformat!("{}  {} (fetch)\n{}  {} (push)",
                    remote.name().unwrap_or("no name"),
                    remote.url().unwrap_or("no url"),
                    remote.name().unwrap_or("no name"),
                    remote.pushurl().or_else(|| remote.url()).unwrap_or(""),
                    ));
                } else {
                    error!("{}", lformat!("no remote"))
                }

            } else {
                error!("{}", lformat!("no remote name"))
            }
        }

    }

    Ok(())
}

/// Command ADD
pub fn git_add(matches: &ArgMatches<'_>) -> Result<()> {
    trace!("git_add {:#?}", matches);
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    let paths = matches_to_paths(matches, &storage)?;

    if matches.is_present("all") {
        if repo.add_all().success() {
            Ok(())
        } else {
            Err("git add did not exit successfully".into())
        }
    } else if matches.is_present("search_term") {

        if repo.add(&paths).success() {
            Ok(())
        } else {
            Err("git add did not exit successfully".into())
        }

    } else {
        Err("Nothing selected".into())
    }
}


/// Command DIFF
pub fn git_diff(matches: &ArgMatches<'_>) -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let paths = matches_to_paths(matches, &storage)?;
    let repo = storage.repository().unwrap();
    let flags = if matches.is_present("staged") {
        vec!["--staged"]
    } else {
        Vec::new()
    };
    if !repo.diff(&paths, &flags).success() {
        return Err("git diff did not exit successfully".into());
    }
    Ok(())
}

/// Command PULL
pub fn git_pull(matches: &ArgMatches<'_>) -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();

    let success = if matches.is_present("rebase") {
        repo.pull_rebase().success()
    } else {
        repo.pull().success()
    };
    if !success {
        return Err("git pull did not exit successfully".into());
    }
    Ok(())
}

/// Command PUSH
pub fn git_push() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.push().success() {
        return Err("git push did not exit successfully".into());
    }
    Ok(())
}

/// Command STASH
pub fn git_stash() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.stash().success() {
        return Err("git stash did not exit successfully".into());
    }
    Ok(())
}

/// Command CLEANUP
pub fn git_cleanup(matches: &ArgMatches<'_>) -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let paths = matches_to_paths(matches, &storage)?;
    let repo = storage.repository().unwrap();
    // TODO implement `.and()` for exitstatus

    if util::really(&format!("Do you really want to reset any changes you made to:\n {:?}\n",
                             paths)) && !(repo.checkout(&paths).success() && repo.clean(&paths).success())
    {
        return Err("clean was not successfull".into());
    }
    Ok(())
}

/// Command STASH POP
pub fn git_stash_pop() -> Result<()> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.stash_pop().success() {
        Err("git stash pop did not exit successfully".into())
    } else {
        Ok(())
    }
}
