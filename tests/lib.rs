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


#[cfg(all(feature="serialization",test))]
mod regression {
    use super::*;
    use asciii::storage::Storable;
    use asciii::BillType;

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

    /// This tests that old projects and new projects produce the exaxt same export format
    #[test]
    fn open_eqjl_projects() {
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
            compare_exports::<Service>(&project1, &project2);
        }
    }

    #[test]
    fn test_document_export() {
        println!("🤷🏽‍♂️");
        for file in list_path_content(Path::new("./tests/eql_projects")) {
            println!("opening {}", file.display());
            let project = Project::open_file(&file).unwrap();

            let template = "./templates/export.tex.hbs";
            let exported_project: Complete = project.export();

            let filled = asciii::document_export::fill_template(&exported_project, BillType::Offer, template).unwrap();
        }

    }
}


#[cfg(test)]
mod taxed_service {
    use super::*;
    use asciii::project::spec::HasEmployees;


    #[test]
    fn services_are_implicitely_zero() {
        let  hours_zerotaxed: &str = "hours: { salary: 8.0, caterers: { unknown: 3 }, tax: 0 }";
        let  hours_untaxed: &str   = "hours: { salary: 8.0, caterers: { unknown: 3 } }";

        let project_untaxed   = Project::from_file_content(&hours_untaxed).unwrap();
        let project_zerotaxed = Project::from_file_content(&hours_zerotaxed).unwrap();

        assert_eq!(
            project_untaxed  .hours().net_wages(),
            project_zerotaxed.hours().net_wages()
            );
    }

    #[test]
    fn services_calculate_gross_wages() {
        let hours_taxed: &str   = r#"
hours: { salary: 8.0, caterers: { unknown: 3 }, tax: 0.19 }
tax: 0.19"#;
        let hours_intaxed: &str = "hours: { salary: 9.52, caterers: { unknown: 3 } }";

        let project_taxed     = Project::from_file_content(&hours_taxed).unwrap();
        let project_intaxed   = Project::from_file_content(&hours_intaxed).unwrap();
        assert_eq!(
            project_taxed    .hours().net_wages(),
            project_intaxed  .hours().net_wages()
            );
    }
}
