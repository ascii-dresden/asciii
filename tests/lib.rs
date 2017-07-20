extern crate asciii;
#[macro_use] extern crate log;
#[macro_use] extern crate pretty_assertions;

use std::fs;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use asciii::project::Project;
//use asciii::project::spec::*;
use asciii::project::export::*;

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
mod regression {
    use super::*;
    use asciii::storage::Storable;

    #[test]
    fn open_old_projects() {
        for file in list_path_content(Path::new("./tests/old_projects")) {
            println!("opening {}", file.display());
            let _project = Project::open_file(&file).unwrap();
        }
    }

    fn compare_exports<T: PartialEq + Debug>(project1: &Project, project2: &Project)
        where asciii::project::Project: asciii::project::export::ExportTarget<T>
    {
            let export1: T = project1.export();
            let export2: T = project2.export();

            assert_eq!(export1, export2);
    }

    #[test]
    fn open_eql_projects() {
        for files in list_path_content(Path::new("./tests/eql_projects")).windows(2) {
            let (file1, file2) = (&files[0], &files[1]);

            println!("opening {}", file1.display());
            let project1 = Project::open_file(&file1).unwrap();

            println!("opening {}", file2.display());
            let project2 = Project::open_file(&file2).unwrap();

            println!("comparing {} with {}", file1.display(), file2.display());
            compare_exports::<Client>(&project1, &project2);
            compare_exports::<Event>(&project1, &project2);
            compare_exports::<Offer>(&project1, &project2);
            compare_exports::<Invoice>(&project1, &project2);
            compare_exports::<Bills>(&project1, &project2);
            compare_exports::<Complete>(&project1, &project2);
            compare_exports::<Hours>(&project1, &project2);
        }
    }


}
