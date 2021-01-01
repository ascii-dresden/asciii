use clap::ArgMatches;
use anyhow::{bail, format_err, Error};

use asciii::{storage, util};
use asciii::project::Project;

use super::matches_to_paths;

/// Command LOG
pub fn git_log(matches: &ArgMatches<'_>) -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let paths = matches_to_paths(matches, &storage)?;
    let repo = storage.repository().unwrap();
    if !repo.log(&paths).success() {
        bail!(format_err!("git log did not exit successfully"));
    } else {
        Ok(())
    }
}

/// Command STATUS
pub fn git_status() -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.status().success() {
        bail!(format_err!("git status did not exit successfully"));
    } else {
        Ok(())
    }
}

/// Command COMMIT
pub fn git_commit() -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.commit().success() {
        bail!(format_err!("git commit did not exit successfully"));
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
pub fn git_remote() -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    storage.repository().unwrap().remote();
    Ok(())
}

/// Command REMOTE
/// exact replica of `git remote -v`
#[cfg(feature = "git_statuses")]
pub fn git_remote() -> Result<(), Error> {
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
                    log::error!("{}", lformat!("no remote"))
                }

            } else {
                log::error!("{}", lformat!("no remote name"))
            }
        }

    }

    Ok(())
}

/// Command ADD
pub fn git_add(matches: &ArgMatches<'_>) -> Result<(), Error> {
    log::trace!("git_add {:#?}", matches);
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    let paths = matches_to_paths(matches, &storage)?;

    if matches.is_present("all") {
        if repo.add_all().success() {
            Ok(())
        } else {
            bail!(format_err!("git add did not exit successfully"));
        }
    } else if matches.is_present("search_term") {

        if repo.add(&paths).success() {
            Ok(())
        } else {
            bail!(format_err!("git add did not exit successfully"));
        }

    } else {
        bail!(format_err!("Nothing selected"));
    }
}


/// Command DIFF
pub fn git_diff(matches: &ArgMatches<'_>) -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let paths = matches_to_paths(matches, &storage)?;
    let repo = storage.repository().unwrap();
    let flags = if matches.is_present("staged") {
        vec!["--staged"]
    } else {
        Vec::new()
    };
    if !repo.diff(&paths, &flags).success() {
        bail!(format_err!("git diff did not exit successfully"));
    }
    Ok(())
}

/// Command PULL
pub fn git_pull(matches: &ArgMatches<'_>) -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();

    let success = if matches.is_present("rebase") {
        repo.pull_rebase().success()
    } else {
        repo.pull().success()
    };
    if !success {
        bail!(format_err!("git pull did not exit successfully"));
    }
    Ok(())
}

/// Command PUSH
pub fn git_push() -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.push().success() {
        bail!(format_err!("git push did not exit successfully"));
    }
    Ok(())
}

/// Command STASH
pub fn git_stash() -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.stash().success() {
        bail!(format_err!("git stash did not exit successfully"));
    }
    Ok(())
}

/// Command CLEANUP
pub fn git_cleanup(matches: &ArgMatches<'_>) -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let paths = matches_to_paths(matches, &storage)?;
    let repo = storage.repository().unwrap();
    // TODO: implement `.and()` for exit status

    if util::really(&format!("Do you really want to reset any changes you made to:\n {:?}\n",
                             paths)) && !(repo.checkout(&paths).success() && repo.clean(&paths).success())
    {
        bail!(format_err!("clean was not successful"));
    }
    Ok(())
}

/// Command STASH POP
pub fn git_stash_pop() -> Result<(), Error> {
    let storage = storage::setup_with_git::<Project>()?;
    let repo = storage.repository().unwrap();
    if !repo.stash_pop().success() {
        bail!(format_err!("git stash pop did not exit successfully"));
    } else {
        Ok(())
    }
}
