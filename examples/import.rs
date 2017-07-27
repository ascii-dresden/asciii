extern crate serde;
extern crate serde_yaml;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate ordered_float;
extern crate num_traits;

use ordered_float::OrderedFloat;
use num_traits::Float;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(remote = "OrderedFloat")]
struct OrderedFloatDef<T: Float>(pub T);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct Number(
    #[serde(with = "OrderedFloatDef")]
    OrderedFloat<f64>
    );

impl From<Number> for f64 {
    fn from (num: Number) -> f64 {
        let Number(ord) = num;
        ord.into_inner()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    client: Client,
    event: Event,
    hours: Hours,
    products: HashMap<ProductDesc, Product>,
    manager: Option<String>,
    invoicer_version: Option<String>,
    template: Option<String>,
    created: Option<String>,
    canceled: Option<String>,
    meta: Option<Meta>,
    tax: Option<Number>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Client {
    title: String,
    first_name: String,
    last_name: String,
    email: Option<String>,
    address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Event {
    name: String,
    location: Option<String>,
    description: Option<String>,
    dates: Vec<Date>
}

#[derive(Debug, Serialize, Deserialize)]
struct Date {
    begin: String,
    end: Option<String>,
    times: Vec<Time>
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
enum ProductDesc {
    Name(String),
    Head{
        name: String,
        price: Number,
        unit: Option<String>,
        tax: Option<Number>
    },

}
impl ProductDesc {
    fn name(&self) -> String {
        match *self {
            ProductDesc::Name(ref name) => name.to_owned(),
            ProductDesc::Head{ref name, .. } => name.clone(),

        }
    }

    fn price(&self) -> Option<f64> {
        match *self {
            ProductDesc::Name(_) => None,
            ProductDesc::Head{ref price, .. } => Some(price.clone().into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Product {
    amount: f64,
    returned: Option<f64>,
    price: Option<f64>,
    sold: Option<f64>,
}


#[derive(Debug, Serialize, Deserialize)]
struct Hours {
    salary: f64,
    caterers: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Time {
    begin: String,
    end: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
struct Meta {
    invoicer_version: String,
    template: String,
    format: String,
}

fn main() {
    let file = include_str!("test.yml");
    let project: Project = serde_yaml::from_str(&file).unwrap();

    //println!("{:#?}\n", project);
    for (product, details) in &project.products {
        println!("{:?}: {}", product.name(), details.price.or(product.price()).unwrap());
    }
}

