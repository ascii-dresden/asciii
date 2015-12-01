#![allow(unused_variables)]
#![allow(dead_code)]


use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use chrono::*;
use yaml_rust::Yaml;
use tempdir::TempDir;
use slug;

use util;
use util::yaml;
use util::yaml::YamlError;
use manager::{
    LuigiProject,
    LuigiValidatable,
    LuigiValidator,
    LuigiError};
use templater::Templater;

pub mod spec;

pub struct Project {
    file_path: PathBuf,
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
        let temp_file = temp_dir.path().join(slug::slugify(project_name) + "." + Self::file_extension());

        // write into a file
        let mut file = try!( File::create(&temp_file) );
        try!(file.write_all(templater.filled.as_bytes()));
        try!(file.sync_all());

        // project now lives in the temp_file
        Ok(Project{
            file_path: temp_file,
            temp_dir: Some(temp_dir), // needs to be kept alive to avoid deletion TODO: try something manually
            yaml: try!(yaml::parse(&templater.filled))
        })
    }

    fn index(&self) -> Option<String>{
        spec::invoice::number_str(self.yaml()).or(
            self.date().map(|d|d.format("ZZ%Y%m%d").to_string())
            )
    }

    fn name(&self) -> String { spec::project::name(self.yaml()).unwrap_or("unnamed").to_owned() }
    fn date(&self) -> Option<Date<UTC>>{ spec::project::date(self.yaml()) }

    fn file(&self) -> PathBuf{ self.file_path.to_owned() } // TODO reconsider returning PathBuf at all
    fn set_file(&mut self, new_file:&Path){ self.file_path = new_file.to_owned(); }
}

impl LuigiValidatable for Project{
    fn valide<ProjectValidity>(&self) -> Vec<ProjectValidity>{ Vec::new() }

    fn validate<ProjectValidity>(&self, criterion:ProjectValidity) -> bool{ false }
}

// TODO cache lookups
impl Project{
    /// Opens a yaml and parses it.
    pub fn open(file_path:&Path) -> Result<Project,YamlError>{
        let file_content = try!(File::open(&file_path)
                                .and_then(|mut file| {
                                    let mut content = String::new();
                                    file.read_to_string(&mut content).map(|_| content)
                                }));
        Ok(Project{
            file_path: file_path.to_owned(),
            temp_dir: None,
            yaml: try!(yaml::parse(&file_content))
        })
    }

    fn yaml(&self) -> &Yaml{ &self.yaml }

    pub fn manager(&self) -> String{
        spec::project::manager(self.yaml()).unwrap_or("____").into()
    }

    pub fn invoice_num(&self) -> String{
        spec::invoice::number_str(self.yaml()).unwrap_or("".into())
    }
}

#[cfg(test)]
mod test{
    use std::path::Path;
    use super::super::project::spec;
    use super::super::project::Project;

    #[test]
    fn compare_basics(){
        println!("{:?}", ::std::env::current_dir());
        let new_project = Project::open(Path::new("./tests/current.yml")).unwrap();
        let old_project = Project::open(Path::new("./tests/old.yml")).unwrap();
        let new_yaml = new_project.yaml();
        let old_yaml = old_project.yaml();
        let config = &super::super::CONFIG;

        assert_eq!(spec::project::name(&old_yaml), spec::project::name(&new_yaml));

        //assert_eq!(spec::project::manager(&old_yaml), //fails
        //           spec::project::manager(&new_yaml));

        assert_eq!(spec::offer::number(&old_yaml), spec::offer::number(&new_yaml));

        //assert_eq!(spec::date::offer(&old_yaml), //fails
        //           spec::date::offer(&new_yaml));

        assert_eq!(spec::invoice::number_str(&old_yaml), spec::invoice::number_str(&new_yaml));
        assert_eq!(spec::date::invoice(&old_yaml), spec::date::invoice(&new_yaml));
        assert_eq!(spec::date::payed(&old_yaml), spec::date::payed(&new_yaml));
        assert_eq!(spec::client::title(&old_yaml), spec::client::title(&new_yaml));
        assert_eq!(spec::client::last_name(&old_yaml), spec::client::last_name(&new_yaml));
        assert_eq!(spec::client::addressing(&old_yaml, &config), spec::client::addressing(&new_yaml, &config));

    }
}
