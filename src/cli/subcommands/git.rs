use clap::ArgMatches;

use asciii::actions::setup_luigi_with_git;
use asciii::util;

use ::cli::execute;
use super::matches_to_paths;

/// Command LOG
pub fn git_log() {
    let luigi = execute(setup_luigi_with_git);
    let repo = luigi.repository().unwrap();
    if !repo.log().success() {
        error!("git log did not exit successfully")
    }
}

/// Command STATUS
pub fn git_status() {
    let luigi = execute(setup_luigi_with_git);
    let repo = luigi.repository().unwrap();
    if !repo.status().success() {
        error!("git status did not exit successfully")
    }
}

/// Command COMMIT
pub fn git_commit() {
    let luigi = execute(setup_luigi_with_git);
    let repo = luigi.repository().unwrap();
    if !repo.commit().success() {
        error!("git commit did not exit successfully")
    }
}

/// Command REMOTE
/// exact replica of `git remote -v`
#[cfg(not(feature="git_statuses"))]
pub fn git_remote() {
    let luigi = execute(setup_luigi_with_git);
    luigi.repository().unwrap().remote();
}

/// Command REMOTE
/// exact replica of `git remote -v`
#[cfg(feature="git_statuses")]
pub fn git_remote() {
    let luigi = execute(setup_luigi_with_git);

    if let Some(r) = luigi.repository() {
        let ref repo = r.repo;

        for remote_name in repo.remotes().unwrap().iter() {

            if let Some(name) = remote_name {

                if let Ok(remote) = repo.find_remote(name) {
                    println!("{}  {} (fetch)\n{}  {} (push)",
                    remote.name().unwrap_or("no name"),
                    remote.url().unwrap_or("no url"),
                    remote.name().unwrap_or("no name"),
                    remote.pushurl().or(remote.url()).unwrap_or(""),
                    );
                } else {
                    println!("no remote")
                }

            } else {
                println!("no remote name")
            }
        }

    }

}

/// Command ADD
pub fn git_add(matches: &ArgMatches) {
    let luigi = execute(setup_luigi_with_git);
    let paths = matches_to_paths(matches, &luigi);
    let repo = luigi.repository().unwrap();
    if !repo.add(&paths).success() {
        error!("git add did not exit successfully")
    }
}


/// Command DIFF
pub fn git_diff(matches: &ArgMatches) {
    let luigi = execute(setup_luigi_with_git);
    let paths = matches_to_paths(matches, &luigi);
    let repo = luigi.repository().unwrap();
    if !repo.diff(&paths).success() {
        error!("git diff did not exit successfully")
    }
}

/// Command PULL
pub fn git_pull(matches: &ArgMatches) {
    let luigi = execute(setup_luigi_with_git);
    let repo = luigi.repository().unwrap();

    let success = if matches.is_present("rebase") {
        repo.pull_rebase().success()
    } else {
        repo.pull().success()
    };
    if !success {
        error!("git pull did not exit successfully")
    }
}

/// Command PUSH
pub fn git_push() {
    let luigi = execute(setup_luigi_with_git);
    let repo = luigi.repository().unwrap();
    if !repo.push().success() {
        error!("git push did not exit successfully")
    }
}

/// Command STASH
pub fn git_stash() {
    let luigi = execute(setup_luigi_with_git);
    let repo = luigi.repository().unwrap();
    if !repo.stash().success() {
        error!("git stash did not exit successfully")
    }
}

/// Command CLEANUP
pub fn git_cleanup(matches: &ArgMatches) {
    let luigi = execute(setup_luigi_with_git);
    let paths = matches_to_paths(matches, &luigi);
    let repo = luigi.repository().unwrap();
    // TODO implement `.and()` for exitstatus

    if util::really(&format!(
            "Do you really want to reset any changes you made to:\n {:?}\n[y|N]", paths)) {

        if !(repo.checkout(&paths).success() && repo.clean(&paths).success()) {
            error!("clean was not successfull");
        }
    }
}

/// Command STASH POP
pub fn git_stash_pop() {
    let luigi = execute(setup_luigi_with_git);
    let repo = luigi.repository().unwrap();
    if !repo.stash_pop().success() {
        error!("git stash pop did not exit successfully")
    }
}
