extern crate asciii;
extern crate pretty_assertions;
#[macro_use] extern crate log;

use std::fs;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use asciii::storage::Storable;
use asciii::project::Project;
// use asciii::project::spec::*;
// use asciii::project::export::*;

/// Basically `ls`, returns a list of paths.
fn list_path_content(path:&Path) -> Vec<PathBuf> {
    if !path.exists() {
        error!("Path does not exist: {}", path.display());
    }

    fs::read_dir(path).unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new("yml")))
        .collect::<Vec<PathBuf>>()
}


#[cfg(test)]
mod acceptance {
    use super::*;

    #[test]
    fn open_old_projects() {
        for file in list_path_content(Path::new("./tests/old_projects")) {
            println!("opening {}", file.display());
            let _project = Project::open_file(&file).unwrap();
        }
    }


}
