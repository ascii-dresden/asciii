extern crate ascii_invoicer;
use std::path::Path;

use ascii_invoicer::project::spec;
use ascii_invoicer::project::Project;

fn main(){
    let new_project = Project::open(Path::new("./examples/current.yml")).unwrap();
    let old_project = Project::open(Path::new("./examples/old.yml")).unwrap();
    let config = &ascii_invoicer::CONFIG;

    for yaml in [old_project.yaml(), new_project.yaml()].iter(){

    println!("Name:     {:?}", spec::project::name(&yaml));
    println!("Manager:  {:?}", spec::project::manager(&yaml));
    println!("Offer:    {:?}", spec::offer::number(&yaml));
    println!("          {:?}", spec::date::offer(&yaml));
    println!("Invoice:  {:?}", spec::invoice::number_str(&yaml));
    println!("          {:?}", spec::date::invoice(&yaml));
    println!("Payed     {:?}", spec::date::payed(&yaml));
    println!("Title:    {:?}", spec::client::title(&yaml));
    println!("LastName: {:?}", spec::client::last_name(&yaml));
    println!("Client:   {:?}", spec::client::addressing(&yaml, config));

    println!("--------------");
    }
}

