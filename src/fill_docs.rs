//! Fills export templates to create tex documents.
//!
//! Haven't decided on a templating engine yet, my own will probably not do.

use std::fmt;
use std::path::Path;
use std::error::Error;
use rustc_serialize::json::{ToJson, Json};
use handlebars::{RenderError, Handlebars, no_escape};

use util;
use project::Project;
use storage::Storage;

custom_derive! {
    #[derive(Debug,
             IterVariants(VirtualFields), IterVariantNames(VirtualFieldNames),
             EnumFromStr
             )]
pub enum Template{
    Document,
    Simple,
    Invalid
}
}

impl<'a> From<&'a str> for Template {
    fn from(s: &'a str) -> Template {
        s.parse::<Template>().unwrap_or(Template::Invalid)
    }
}

#[derive(Debug)]
pub enum FillError {
    RenderError(RenderError),
    InvalidTemplate,
}

impl Error for FillError {
    fn description(&self) -> &str {
        match *self {
            FillError::RenderError(ref inner) => inner.description(),
            FillError::InvalidTemplate => "Invalid Template",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            FillError::RenderError(ref inner) => Some(inner),
            FillError::InvalidTemplate => None,
        }
    }
}

impl fmt::Display for FillError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.cause() {
            None => write!(f, "{}", self.description(),),
            Some(cause) => write!(f, "{}", cause),
        }
    }
}

// All you need to make try!() fun again
impl From<RenderError> for FillError {
    fn from(he: RenderError) -> FillError {
        FillError::RenderError(he)
    }
}


/// Sets up an instance of `Storage`.
fn setup_luigi() -> Storage<Project> {
    let working = ::CONFIG.get_str("dirs/working")
        .expect("Faulty config: dirs/working does not contain a value");
    let archive = ::CONFIG.get_str("dirs/archive")
        .expect("Faulty config: dirs/archive does not contain a value");
    let templates = ::CONFIG.get_str("dirs/templates")
        .expect("Faulty config: dirs/templates does not contain a value");
    Storage::new(util::get_storage_path(), working, archive, templates).unwrap()
}

struct PackData<'a, T: 'a + ToJson> {
    document: &'a T,
    storage: Storage<Project>,
    is_invoice:bool
}


impl<'a, T> ToJson for PackData<'a, T>
    where T: ToJson
{
    fn to_json(&self) -> Json {
        Json::Object(btreemap!{
            String::from("document")   => self.document.to_json(),
            String::from("storage")    => self.storage.to_json(),
            String::from("is_invoice") => self.is_invoice.to_json()
        })
    }
}

fn pack_data<E: ToJson>(document: &E, is_invoice:bool) -> PackData<E> {
    PackData {
        document: document,
        storage: setup_luigi(),
        is_invoice:is_invoice
    }
}

use handlebars::{Context, Helper, RenderContext};
fn inc_helper(_: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    // just for example, add error check for unwrap
    let param = h.param(0).unwrap().value();
    let rendered = format!("{}", param.as_u64().unwrap() + 1);
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

/// Takes a `T:ToJson` and a template path and does it's thing.
///
/// Returns path to created file, potenially in a `tempdir`.
// pub fn fill_template<E:ToJson>(document:E, template_file:&Path) -> PathBuf{
pub fn fill_template<E: ToJson>(document: &E, is_invoice:bool, template: Template) -> Result<String, FillError> {

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    handlebars.register_helper("inc", Box::new(inc_helper));

    handlebars.register_template_file(
        "document",
        Path::new("./templates/document.tex.hbs"))
        .unwrap();

    handlebars.register_template_file(
        "simple",
        Path::new("./templates/simple.hbs"))
        .unwrap();

    let packed = pack_data(document, is_invoice);

    match template {
        Template::Document => {
            handlebars.register_escape_fn(|data| data.replace("\n", r#"\newline "#));
            handlebars.render("document", &packed)
                      .map(|r| r.replace("<", "{")
                                .replace(">", "}"))
        }
        Template::Simple => handlebars.render("simple", &packed),
        Template::Invalid => return Err(FillError::InvalidTemplate),
    }
    .map_err(FillError::RenderError)
}
