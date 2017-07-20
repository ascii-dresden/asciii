extern crate asciii;
#[macro_use] extern crate log;
#[macro_use] extern crate pretty_assertions;

use std::fs;
use std::ffi::OsStr;
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
mod acceptance {
    use super::*;
    use asciii::storage::Storable;

    #[test]
    fn open_old_projects() {
        for file in list_path_content(Path::new("./tests/old_projects")) {
            println!("opening {}", file.display());
            let _project = Project::open_file(&file).unwrap();
        }
    }

    #[test]
    fn open_eql_projects() {
        for files in list_path_content(Path::new("./tests/eql_projects")).windows(2) {
            let (file1,file2) = (&files[0], &files[1]);

            type Export = Invoice;

            println!("opening {}", file1.display());
            let project1 = Project::open_file(&file1).unwrap();
            let export1: Export = project1.export();

            println!("opening {}", file2.display());
            let project2 = Project::open_file(&file2).unwrap();
            let export2: Export = project2.export();

            println!("comparing {} with {}", file1.display(), file2.display());
            //assert_eq!(export1.offer, export2.offer);
            //assert_eq!(export1.invoice, export2.invoice);
            assert_eq!(export1, export2);
        }
    }


}
