extern crate asciii;
#[macro_use] extern crate log;

use std::env;
use std::path::Path;

use asciii::storage::Storable;
use asciii::project::Project;

fn main() {
    asciii::util::setup_log();

    if let Some(project_file) = env::args().nth(1) {

        let project = Project::open_file(Path::new(&project_file));
        let output= Path::new("./foo.pdf");

        debug!("{:?} -> {:?}", project_file, output.display());

        unimplemented!();


    } else {
        error!("please pass in a projectfile");
    }

}

