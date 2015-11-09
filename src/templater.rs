use std::io;
use std::io::Read;
use std::fs::File;
use std::path::PathBuf;
use std::collections::HashMap;

use keyword_replacement::IsKeyword;

/// Simple templating module
pub struct Templater{
    /// path to used template file
    pub path: PathBuf,

    /// content of template file after reading
    pub template: String,

    /// content of filled template
    pub filled: String,
}

// TODO work only on yaml r-values
impl Templater{

    pub fn new (path:&str) -> Result<Templater, io::Error> {
        let template = try!(File::open(&path)
            .and_then(|mut file| {
                let mut content = String::new();
                file.read_to_string(&mut content).map(|_| content)
            }));

        Ok(Templater{
            path:PathBuf::from(path),
            template:template,
            filled: String::new()
        })
    }

    pub fn finalize(&mut self) -> Templater {
        Templater{
            path : self.path.to_owned(),
            template: self.template.to_owned(),
            filled: self.filled.to_owned()
        }
    }

    pub fn fill_in_data(&mut self, data: &HashMap<&str,&str>) -> &mut Templater {
        self.fill_template(|keyword| match data.get(keyword){
            Some(content) => content.to_string(),
            None => format!("__{}__", keyword)
        })
    }

    pub fn fill_template<F>(&mut self, closure: F) -> &mut Templater
        where F:Fn(&str) -> String {
        self.filled = self.template.map_keywords(closure);
        self
    }
}
