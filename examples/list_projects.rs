use asciii;

use asciii::project::Project;

fn main() {
    let storage = asciii::storage::setup::<Project>().unwrap();
    for project in storage.open_working_dir_projects().unwrap() {
        println!("{:#?}", project);
    }
}
