extern crate asciii;
#[macro_use] extern crate log;

use std::env;
use std::path::Path;

use asciii::project::Project;
use asciii::document_export::{project_file_to_doc};

fn main() {
    asciii::util::setup_log();

    let project_file = Path::new("/home/hendrik/ascii/caterings/working/thomaskuehn/thomaskuehn.yml");

    //let project_file = env::args().nth(1);

    let project = Project::open(&project_file);

    debug!("project_file {:?}", project_file);

    let output= Path::new("./foo.pdf");


    //project_file_to_doc(&project_file, "document", None, Some(output), false, false).unwrap();



}

