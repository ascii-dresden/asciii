#![allow(unused_variables)]
#![allow(dead_code)]

use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use chrono::*;
use yaml_rust::Yaml;
use tempdir::TempDir;
use slug;
use currency::Currency;

use util::yaml;
use storage::{Storable, StorageResult, StorageError};
use templater::Templater;

pub mod product;
pub mod spec;
use self::spec::{SpecResult, VirtualField};
use self::spec::products::{ProductResult};

pub struct Project {
    file_path: PathBuf,
    temp_dir: Option<TempDir>,
    yaml: Yaml
}

impl From<yaml::YamlError> for StorageError {
    fn from(yerror: yaml::YamlError) -> StorageError{ StorageError::ParseError(yerror) }
}

impl Storable for Project{
    fn from_template(project_name:&str,template:&Path) -> Result<Project,StorageError> {
        let template_name = template.file_stem().unwrap().to_str().unwrap();

        let event_date = (Local::today() + Duration::days(14)).format("%d.%m.%Y").to_string();
        let created_date = Local::today().format("%d.%m.%Y").to_string();

        // fill template with this data
        let data = &hashmap!{
            "VERSION"       => "3.0.0-alpha",
            "TEMPLATE"      => template_name,
            "PROJECT-NAME"  => project_name,
            "DATE-EVENT"    => &event_date,
            "DATE-CREATED"  => &created_date,
            "SALARY"        => "8.0", //::CONFIG.get_as_str("defaults/salary"),
            "manager"       => ::CONFIG.get_str("manager_name")
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

    fn prefix(&self) -> Option<String>{
        self.invoice_num()
    }

    fn index(&self) -> Option<String>{
        if let Some(date) = self.date(){
            spec::invoice::number_str(self.yaml())
                .map(|num| format!("{1}{0}", date.format("%Y%m%d").to_string(),num))
                .or_else(||Some(date.format("zzz%Y%m%d").to_string()))
        } else {
            None
        }
    }

    fn name(&self) -> String {
        spec::project::name(self.yaml())
            .unwrap_or("unnamed").to_owned()
    }

    fn date(&self) -> Option<Date<UTC>>{ spec::project::date(self.yaml()) }

    fn file(&self) -> PathBuf{ self.file_path.to_owned() } // TODO reconsider returning PathBuf at all
    fn set_file(&mut self, new_file:&Path){ self.file_path = new_file.to_owned(); }

    /// Opens a yaml and parses it.
    fn open(file_path:&Path) -> StorageResult<Project>{
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

    /// Checks against a certain key-val pair.
    fn matches_filter(&self, key: &str, val: &str) -> bool{
        self.get(key).map(|c|{
            c.to_lowercase().contains(&val.to_lowercase())
        }).unwrap_or(false)
    }

    /// UNIMPLEMENTED: Checks against a certain search term.
    ///
    /// TODO compare agains InvoiceNumber, ClientFullName, Email, event/name, invoice/official Etc
    fn matches_search(&self, term: &str) -> bool{
        // may use matches_filter internally
        unimplemented!()
    }
}

impl Project{
    pub fn yaml(&self) -> &Yaml{ &self.yaml }

    /// wrapper around yaml::get() with replacement
    pub fn get<'a>(&self, path:&str) -> Option<String>{
        VirtualField::from(path).get(self).or_else(||
            yaml::get_as_string(self.yaml(),path)
        )
    }

    pub fn manager(&self) -> String{
        spec::project::manager(self.yaml()).unwrap_or("").into()
    }

    pub fn canceled(&self) -> bool{
        spec::project::canceled(self.yaml())
    }

    pub fn canceled_string(&self) -> &'static str{
        if spec::project::canceled(self.yaml()){"canceled"}
        else {""}
    }

    pub fn invoice_num(&self) -> Option<String>{
        spec::invoice::number_str(self.yaml())
    }

    /// Minimum correctness.
    ///
    /// Ready to send an **offer** to the client.
    pub fn valid_stage1(&self) -> SpecResult{
        spec::offer::validate(&self.yaml)
    }

    /// Valid project
    ///
    /// Ready to send an **invoice** to the client.
    pub fn valid_stage2(&self) -> SpecResult{
        spec::invoice::validate(&self.yaml)
    }

    /// Completely done and in the past.
    ///
    /// Ready to be **archived**.
    pub fn valid_stage3(&self) -> SpecResult{
        spec::archive::validate(&self.yaml)
    }

    pub fn age(&self) -> Option<i64> {
        self.date().map(|date| (Local::today() - date).num_days() )
    }

    pub fn invoice_items(&self) -> ProductResult<Vec<product::InvoiceItem>> {
        spec::products::all(self.yaml())
    }

    pub fn wages(&self) -> Option<Currency> {
        if let (Some(total), Some(salary)) = (spec::hours::total(&self.yaml), spec::hours::salary(&self.yaml)){
            Some(total * salary)
        } else{None}
    }

    pub fn sum_offered(&self) -> Option<Currency> {
        spec::products::all(self.yaml()).ok() .map(|products| spec::products::sum_offered(&products))
    }

    pub fn sum_sold(&self) -> Option<Currency> {
        spec::products::all(self.yaml()).ok()
            .map(|products| spec::products::sum_sold(&products))
    }

    pub fn tax_offered(&self) -> Option<Currency> {
        spec::products::all(self.yaml()).ok()
            .map(|products| spec::products::sum_offered(&products))
            .map(|sum| sum * 0.19)
    }

    pub fn tax_sold(&self) -> Option<Currency> {
        spec::products::all(self.yaml()).ok()
            .map(|products| spec::products::sum_sold(&products))
            .map(|sum| sum * 0.19)
    }

    pub fn sum_sold_and_taxes(&self) -> Option<Currency> {
        if let (Some(wages), Some(tax), Some(sum)) = (self.wages(), self.tax_sold(), self.sum_sold()){
            Some(sum+tax)
        } else{ None }
    }

    pub fn sum_sold_and_wages(&self) -> Option<Currency> {
        if let (Some(wages), Some(tax), Some(sum)) = (self.wages(), self.tax_sold(), self.sum_sold()){
            Some(wages+sum+tax)
        } else{ None }
    }
}

#[cfg(test)]
mod test{
    use std::path::Path;
    use ::project::spec;
    use ::project::Project;
    use ::storage::Storable;

    #[test]
    fn compare_basics(){
        println!("{:?}", ::std::env::current_dir());
        let new_project = Project::open(Path::new("./tests/current.yml")).unwrap();
        let old_project = Project::open(Path::new("./tests/old.yml")).unwrap();
        let new_yaml = new_project.yaml();
        let old_yaml = old_project.yaml();
        let config = &::CONFIG;

        assert_eq!(spec::project::name(&old_yaml), spec::project::name(&new_yaml));

        //assert_eq!(spec::project::storage(&old_yaml), //fails
        //           spec::project::storage(&new_yaml));

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
