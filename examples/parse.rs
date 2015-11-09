#![allow(dead_code)]
#![allow(unused_variables)]

extern crate yaml_rust;
extern crate chrono;
extern crate ascii_invoicer;

use chrono::UTC;

use ascii_invoicer::templater::Templater;

fn foo_it(_keyword:&str) -> String {
    match _keyword{
        "MORE" => "more is good".into(),
        "VERSION" => "3.0".into(),
        "MANAGER" => "Hendrik Sollich".into(),
        "TEMPLATE" => "invoicer-rs-default".into(),
        "DATE-CREATED" => UTC::today().format("%d.%m.%Y").to_string(),
        _ => format!("__{}__", _keyword)
    }
}


use std::collections::HashMap;
fn main(){
    let mut t = Templater::new("./templates/default.tyml").unwrap();
    let mut data = HashMap::new();
    data.insert("MANAGER", "Hendrik Sollich");
    //t.fill_template(foo_it);
    t.fill_in_data(&data);
    println!("{}", t.filled);
}
