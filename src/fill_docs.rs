//! Fills export templates to create tex documents.
//!
//! Haven't decided on a templating engine yet, my own will probably not do.

use std::path::Path;
use rustc_serialize::json::{ToJson, Json};
use handlebars::{RenderError, Handlebars, no_escape};

use util;
use project::Project;
use storage::Storage;

/// Sets up an instance of `Storage`.
/// TODO isn't this redundant
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

fn path_helper(_: &Context, _: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let rendered = rc.get_local_path_root().to_string();
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

fn count_helper(_: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let count = h.param(0).unwrap().value().as_array().map_or(0, |a|a.len());
    //println!("count_helper{:?}", param);
    let rendered = format!("{}", count);
    try!(rc.writer.write(rendered.into_bytes().as_ref()));
    Ok(())
}

/// Takes a `T:ToJson` and a template path and does it's thing.
///
/// Returns path to created file, potenially in a `tempdir`.
// pub fn fill_template<E:ToJson>(document:E, template_file:&Path) -> PathBuf{
pub fn fill_template<E: ToJson, P:AsRef<Path>>(document: &E, is_invoice:bool, template_path: P) -> Result<String, RenderError> {

    let mut handlebars = Handlebars::new();

    handlebars.register_escape_fn(no_escape);
    handlebars.register_escape_fn(|data| data.replace("\n", r#"\newline "#));
    handlebars.register_helper("inc",   Box::new(inc_helper));
    handlebars.register_helper("path",  Box::new(path_helper));
    handlebars.register_helper("count", Box::new(count_helper));

    handlebars.register_template_file("document", template_path).unwrap();

    let packed = pack_data(document, is_invoice);

    handlebars.render("document", &packed)
              .map(|r| r.replace("<", "{")
                        .replace(">", "}")
                  )
}


