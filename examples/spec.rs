extern crate asciii;
use std::path::Path;

use asciii::project::spec;
use asciii::project::Project;
use asciii::storage::Storable;

fn main(){
    let config = &asciii::CONFIG;

    for project in [
    Project::open(Path::new("./examples/pfeffer.yml")).unwrap(),
    Project::open(Path::new("./examples/current.yml")).unwrap(),
    Project::open(Path::new("./examples/old.yml")).unwrap()
].iter(){
        let yaml = project.yaml();
        println!("Index:     {:?}", project.index());
        println!("Canceled   {:?}", project.canceled());
        println!("Date:      {:?}", project.date());
        println!("Name:      {:?}", spec::project::name(&yaml));
        println!("Manager:   {:?}", spec::project::manager(&yaml));
        println!("Offer:     {:?}", spec::offer::number(&yaml));
        println!("           {:?}", spec::date::offer(&yaml));
        println!("Invoice:   {:?}", spec::invoice::number_str(&yaml));
        println!("           {:?}", spec::date::invoice(&yaml));
        println!("Payed      {:?}", spec::date::payed(&yaml));
        println!("Title:     {:?}", spec::client::title(&yaml));
        println!("FirstName: {:?}", spec::client::first_name(&yaml));
        println!("LastName:  {:?}", spec::client::last_name(&yaml));
        println!("Client:    {:?}", spec::client::addressing(&yaml, config));
        println!("--------------");
        println!("Products:  {:#?}", spec::products::all(&yaml).unwrap());
        println!("--------------");
    }

    //println!("Products: {:#?}", spec::products::all(new_project.yaml()));
}
