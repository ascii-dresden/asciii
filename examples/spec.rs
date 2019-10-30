use std::path::Path;

use asciii::project::spec::*;
use asciii::project::Project;
use asciii::storage::Storable;

fn main() {
    for project in [
        Project::open_file(Path::new("./examples/current.yml")).unwrap(),
        Project::open_file(Path::new("./examples/old.yml")).unwrap(),
    ]
    .iter()
    {
        println!("Index:     {:?}", project.index());
        println!("Canceled   {:?}", project.canceled());
        println!("Date:      {:?}", project.event_date());
        println!("Name:      {:?}", project.name());
        println!("Manager:   {:?}", project.responsible());
        println!("Offer:     {:?}", project.offer().number());
        println!("           {:?}", project.offer().date());
        println!("Invoice:   {:?}", project.invoice().number_str());
        println!("           {:?}", project.invoice().date());
        println!("Payed      {:?}", project.payed_date());
        println!("Title:     {:?}", project.client().title());
        println!("FirstName: {:?}", project.client().first_name());
        println!("LastName:  {:?}", project.client().last_name());
        println!("Client:    {:?}", project.client().addressing());
        println!("--------------");
        // let (_offer, invoice) = spec::billing::bills().unwrap();
        // println!("Products:  {:#?}", invoice.as_items().iter().map(|item|format!("{:?}",item)).collect::<Vec<_>>());
        println!("--------------");
        println!(
            "hours:     {:?}h * {}",
            project.hours().total_time(),
            project
                .hours()
                .salary()
                .map(|c| c.postfix().to_string())
                .ok()
                .unwrap_or_else(|| String::from("0€"))
        );
        println!("caterers:  {:?}", project.hours().employees_string());
        println!("\n\n\n");
    }
}
