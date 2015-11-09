#![allow(dead_code)]

use yaml_rust::{Yaml,YamlLoader};

use chrono::*;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use filter;

enum ProductUnit {
    Piece, Liter, Hour, Kilogramm, Gramm
}

struct Product {
    pub name: String,
    pub unit: ProductUnit,
    pub tax: f64,
    pub price: f64 //TODO make this a currency
}

impl Product {
    fn cost_before_tax()
       // -> f64
    {}

    fn cost_after_tax()
       // -> f64
    {}
}

struct InvoiceItem {
    pub amount_offered: usize,
    pub amount_sold: usize,
    pub item: Product
}


struct Customer {
    pub first_name: String,
    pub last_name: String,
    pub email: String, // TODO replace with e.g. `crate:emailaddress`
}

struct Event {
    pub start:DateTime<UTC>,
    pub end:DateTime<UTC>,

}

#[derive(Debug)]
pub struct Project {
    path: Box<PathBuf>,
    yaml: Vec<Yaml>,
    //customer: Customer, //TODO
    //dates: Vec<Event>,  //TODO
    manager: String
}


impl Project {
    pub fn from_yaml_file(path:&str) -> Project
    {
        let path = PathBuf::from(path);
        let mut file = match File::open(path.as_path())
        {
            Err(but_why)  => panic!("Didn't even open the file right, because: {}", but_why),
            Ok(file) => file
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", path.display(), Error::description(&why)),
            Ok(_) => () //print!("{} contains:\n{}", display, s),
        }

        let yaml_doc = YamlLoader::load_from_str(&s).unwrap();

        Project
        {
            path: Box::new(path),
            manager: yaml_doc[0]["manager"].as_str().expect("\"manager\" missing").to_owned(),
            // do this last!
            yaml: yaml_doc,
        }
    }

    // for testing
    pub fn filter_all(&self)
    {
        println!("date::offer {:?}", filter::date::offer(&self.yaml[0]));
        println!("date::invoice {:?}", filter::date::invoice(&self.yaml[0]));
        println!("date::payed {:?}", filter::date::payed(&self.yaml[0]));
        println!("date::created {:?}", filter::date::created(&self.yaml[0]));
    }

    pub fn created(&self) -> Date<UTC>
    {
        filter::date::created(&self.yaml[0]).unwrap()
    }

    pub fn manager(&self) -> String
    {
        self.yaml[0]["manager"].as_str().expect("\"manager\" missing").to_owned()
    }

}



//#[test]
//fn it_works() {
//    let p = Project::from_yaml_file("./test.yml");
//    p.filter_all();
//    println!("{:?}", p);
//}
