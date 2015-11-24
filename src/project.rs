#![allow(unused_variables)]
#![allow(dead_code)]

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use chrono::*;
use yaml_rust::Yaml;
use pad::{PadStr,Alignment};
use tempdir::TempDir;

use util;
use yaml;
use manager::{LuigiProject, LuigiValidator, LuigiError};
use yaml::YamlError;
use templater::Templater;

pub struct Project {
    pub path: PathBuf,
    temp_dir: Option<TempDir>,
    yaml: Yaml
}

enum ProjectValidity{
    TemplateFilled,
    Offer,
    Invoice,
    Payed,
    Archive
}

impl LuigiValidator for ProjectValidity{}

impl From<yaml::YamlError>  for LuigiError {
    fn from(yerror: yaml::YamlError) -> LuigiError{ LuigiError::ParseError }
}

//#[derive(Debug)]
//pub struct ProjectOldFormat { yaml: Yaml } // implemented differently

impl LuigiProject for Project{
    fn new(project_name:&str,template:&Path) -> Result<Project,LuigiError> {
        let template_name = template.file_stem().unwrap().to_str().unwrap();

        // fill template with this data
        let data = &hashmap!{
            "VERSION"       => "3.0.0-alpha",
            "TEMPLATE"      => template_name,
            "PROJECT-NAME"  => project_name,
            "DATE-EVENT"    => "11.11.2011",
            "DATE-CREATED"  => "11.11.2011",
            "SALARY"        => "8.0", //super::CONFIG.get_as_str("defaults/salery"),
            "MANAGER"       => super::CONFIG.get_str("manager_name")
        };

        // fills the template
        let templater = Templater::new(template)
            .unwrap()
            .fill_in_data(data)
            .finalize();

        // generates a temp file
        let temp_dir  = TempDir::new(&project_name).unwrap();
        let temp_file = temp_dir.path().join(project_name);

        // write into a file
        let mut file = try!( File::create(&temp_file) );
        try!(file.write_all(templater.filled.as_bytes()));
        try!(file.sync_all());

        // project now lives in the temp_file
        Ok(Project{
            path: temp_file,
            temp_dir: Some(temp_dir), // needs to be kept alive to avoid deletion TODO: try something manually
            yaml: try!(yaml::parse(&templater.filled))
        })
    }

    fn index(&self) -> String{
        match yaml::get_int(&self.yaml, "invoice/number"){
            Some(num) => num.to_string().pad_to_width_with_alignment(3,Alignment::Right),
            None => "   ".to_owned()
        }
    }

    fn name<'a>(&'a self) -> &'a str{
        self.y_str("event/name")
    }

    fn date(&self) -> Date<UTC>{
        let date_str = yaml::get_str(&self.yaml, "event/date").or(
                       yaml::get_str(&self.yaml, "created"))
            .unwrap_or("01.01.0000");
        util::parse_fwd_date(date_str)
    }

    fn path(&self) -> PathBuf{
        self.path.to_owned()
    }

    fn file_extension() -> &'static str {"yml"}

    fn valide<ProjectValidity>(&self) -> Vec<ProjectValidity>{ Vec::new() }

    fn validate<ProjectValidity>(&self, criterion:ProjectValidity) -> bool{ false }
}

// TODO cache lookups
impl Project{
    /// Opens a yaml and parses it.
    pub fn open(path:&Path) -> Result<Project,YamlError>{
        let file_content = try!(File::open(&path)
                                .and_then(|mut file| {
                                    let mut content = String::new();
                                    file.read_to_string(&mut content).map(|_| content)
                                }));
        Ok(Project{
            path: path.to_owned(),
            temp_dir: None,
            yaml: try!(yaml::parse(&file_content))
        })
    }

    pub fn manager(&self) -> String{
        yaml::get_str(&self.yaml, "manager").unwrap_or("").to_owned()
    }

    fn y_str<'a>(&'a self, path:&str) -> &'a str{
        // TODO benchmark all these yaml lookups
        // TODO perhaps replace Path parsing with simpler splitting
        // TODO replace a bunch of this with compile time macros
        yaml::get_str(&self.yaml, &path).unwrap_or("")
    }
}

//#[test]
//fn it_works() {
//    let p = Project::from_yaml_file("./test.yml");
//    p.filter_all();
//    println!("{:?}", p);
//}
