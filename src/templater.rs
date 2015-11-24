use std::io;
use std::io::Read;
use std::fs::File;
use std::path::{Path,PathBuf};
use std::collections::HashMap;

use util::IsKeyword;

/// Simple templating module
#[derive(Debug)]
pub struct Templater{
    /// path to used template file
    pub path: PathBuf,

    /// content of template file after reading
    pub original: String,

    /// content of filled template
    pub filled: String,
}

// TODO make templater work only on yaml r-values
impl Templater{
    pub fn new (path:&Path) -> Result<Templater, io::Error> {
        let template = try!(File::open(&path)
            .and_then(|mut file| {
                let mut content = String::new();
                file.read_to_string(&mut content).map(|_| content)
            }));

        Ok(Templater{
            path:PathBuf::from(path),
            original:template,
            filled: String::new()
        })
    }

    pub fn finalize(&mut self) -> Templater {
        self.to_owned()
    }

    pub fn fill_in_data(&mut self, data: &HashMap<&str,&str>) -> &mut Templater {
        self.fill_template(|keyword| match data.get(keyword){
            Some(content) => content.to_string(),
            None => format!("__{}__", keyword)
        })
    }

    pub fn list_keywords(&self) -> Vec<String>{
        self.original.list_keywords()
    }

    pub fn fill_template<F>(&mut self, closure: F) -> &mut Templater
        where F:Fn(&str) -> String {
        self.filled = self.original.map_keywords(closure);
        self
    }
}

use std::borrow::ToOwned;
impl ToOwned for Templater{
    type Owned = Templater;
    fn to_owned(&self) -> Templater {
        Templater{
            path :    self.path.to_owned(),
            original: self.original.to_owned(),
            filled:   self.filled.to_owned()
        }
    }
}
