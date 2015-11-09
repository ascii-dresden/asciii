#![allow(dead_code)]
#![allow(unused_variables)]

extern crate yaml_rust;
extern crate chrono;
extern crate ascii_invoicer;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use chrono::UTC;

use yaml_rust::{Yaml,YamlLoader};

use ascii_invoicer::IsKeyword;

fn read_file(path:&str) -> Result<String,std::io::Error> {
    File::open(PathBuf::from(path))
        .and_then(|mut file| {
            let mut content = String::new();
            file.read_to_string(&mut content).map(|_| content)
        })
}


//pub fn wait_for_action<F>(self, invokation_closure:F) where F:FnOnce(&str)
fn walk_doc(yaml:&Yaml, path: &PathBuf) {
    match yaml{
        &Yaml::Hash(ref more) => for (k,v) in more.iter(){
            walk_doc(&v, &path.join(k.as_str().unwrap()))
        },
        &Yaml::String(ref string) => { string.get_keyword().map(|m| println!("{:?}: {:?}", path, m));},
        _ => ()
    }
}

fn foo_it(_keyword:&str) -> String {
    match _keyword{
        "MORE" => "more is good".into(),
        "VERSION" => "3.0".into(),
        "TEMPLATE" => "invoicer-rs-default".into(),
        "DATE-CREATED" => UTC::today().format("%d.%m.%Y").to_string(),
        _ => format!("__{}__", _keyword)
    }
}

fn main(){
    if let Ok(file_content) = read_file("./test/template1.tyml"){
        let yaml_doc = YamlLoader::load_from_str(&file_content/*.map_keywords(foo_it)*/).unwrap();
        walk_doc(&yaml_doc[0], &PathBuf::from("/"));

        //println!("{:#?}", &yaml_doc);

    }
}
