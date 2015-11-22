#![allow(dead_code)]

use yaml_rust::Yaml;
use chrono::*;
use std::path::{Path, PathBuf};
use util;
use yaml;
use yaml::YamlError;
use manager::LuigiProject;
use pad::{PadStr,Alignment};


#[derive(Debug)]
pub struct Project { path: PathBuf, yaml: Yaml }

//#[derive(Debug)]
//pub struct ProjectOldFormat { yaml: Yaml } // implemented differently

impl LuigiProject for Project{
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
}


// TODO cache lookups
impl Project{

    pub fn open(path:&Path) -> Result<Project,YamlError>{
        Ok(Project{
            path: path.to_owned(),
            yaml: try!(yaml::open_yaml(&path))
        })
    }

    pub fn manager(&self) -> String{
        yaml::get_str(&self.yaml, "manager").unwrap_or("").to_owned()
    }

    fn y_str<'a>(&'a self, path:&str) -> &'a str{
        yaml::get_str(&self.yaml, &path).unwrap_or("")
    }

}

//#[test]
//fn it_works() {
//    let p = Project::from_yaml_file("./test.yml");
//    p.filter_all();
//    println!("{:?}", p);
//}
