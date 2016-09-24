extern crate asciii;
use std::path::Path;

use asciii::project::spec;
use asciii::project::Project;
use asciii::storage::Storable;

fn main() {

    for project in [Project::open_file(Path::new("./examples/current.yml")).unwrap(),
                    Project::open_file(Path::new("./examples/old.yml")).unwrap()]
        .iter() {
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
        println!("Client:    {:?}", spec::client::addressing(&yaml));
        println!("--------------");
        //let (_offer, invoice) = spec::billing::bills(&yaml).unwrap();
        //println!("Products:  {:#?}", invoice.as_items().iter().map(|item|format!("{:?}",item)).collect::<Vec<_>>());
        println!("--------------");
        println!("hours:     {:?}h * {}", spec::hours::total(&yaml), spec::hours::salary(&yaml).map(|c|c.postfix().to_string()).unwrap_or_else(||String::from("0€")));
        println!("caterers:  {:?}", spec::hours::caterers(&yaml));
        println!("\n\n\n");
    }

    // println!("Products: {:#?}", spec::products::all(new_project.yaml()));
}
