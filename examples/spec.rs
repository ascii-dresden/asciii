extern crate asciii;
extern crate yaml_rust as yaml;
use std::path::Path;
use std::result::Result;
use std::fs::File;
use std::io::prelude::*;

use asciii::project::spec;
use asciii::project::Project;
use asciii::storage::Storable;

use yaml::*;

fn _main() {
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
        // let (_offer, invoice) = spec::billing::bills(&yaml).unwrap();
        // println!("Products:  {:#?}", invoice.as_items().iter().map(|item|format!("{:?}",item)).collect::<Vec<_>>());
        println!("--------------");
        println!("hours:     {:?}h * {}", spec::hours::total(&yaml), spec::hours::salary(&yaml) .map(|c| c.postfix().to_string()).unwrap_or_else(|| String::from("0€")));
        println!("caterers:  {:?}", spec::hours::caterers(&yaml));
        println!("\n\n\n");
    }

    // println!("Products: {:#?}", spec::products::all(new_project.yaml()));
}

/// Ruby like API to yaml-rust.
pub fn yaml_parse( file_content:&str ) -> Result<Yaml, ScanError> {
    Ok(
        try!(YamlLoader::load_from_str(&file_content))
        .get(0)
        .map(|i|i.to_owned())
        .unwrap_or_else(||Yaml::from_str("[]"))
      )
}

struct TestProject {
    yaml: Yaml
}

impl TestProject {
    /// Opens a yaml and parses it.
    fn open_file(file_path:&Path)  -> Result<TestProject,ScanError>{
        let file_content = try!(File::open(&file_path)
                                .and_then(|mut file| {
                                    let mut content = String::new();
                                    file.read_to_string(&mut content).map(|_| content)
                                }));
        Ok(TestProject{
            yaml: try!(yaml_parse(&file_content)),
        })
    }

}

fn main() {
    for project in [TestProject::open_file(Path::new("./examples/current.yml")).unwrap(),
                    TestProject::open_file(Path::new("./examples/old.yml")).unwrap()]
        .iter() {
    }
}
