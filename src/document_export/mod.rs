//! Fills export templates to create tex documents.
//!
//! Haven't decided on a templating engine yet, my own will probably not do.

use std::io;
use std::fmt;
use std::time;
use std::fs;
use std::path::{Path,PathBuf};

use open;
use rustc_serialize::json::{ToJson, Json};
use handlebars::{RenderError, Handlebars, no_escape};
use handlebars::{Context, Helper, RenderContext};

use util;
use project;
use project::Project;
use storage::error::StorageError;
use storage::{self,Storage,StorageDir,Storable, StorageSelection};

pub mod error {
    use super::*;

    error_chain!{
        types {
            Error, ErrorKind, ResultExt, Result;
        }

        links { }

        foreign_links {
            Io(io::Error);
            Fmt(fmt::Error);
            Time(time::SystemTimeError);
            Handlebar(RenderError);
            Project(project::error::Error);
            Storage(StorageError);
        }

        errors {
            NoPdfCreated{ description("No Pdf Created") }
            NothingToDo{ description("Nothing to do") }
        }
    }
}

use self::error::*;

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
        storage: storage::setup().unwrap(),
        is_invoice:is_invoice
    }
}

fn inc_helper(_: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> ::std::result::Result<(), RenderError> {
    // just for example, add error check for unwrap
    let param = h.param(0).unwrap().value();
    let rendered = format!("{}", param.as_u64().unwrap() + 1);
    rc.writer.write_all(rendered.into_bytes().as_ref())?;
    Ok(())
}

fn count_helper(_: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> ::std::result::Result<(), RenderError> {
    let count = h.param(0).unwrap().value().as_array().map_or(0, |a|a.len());
    //println!("count_helper{:?}", param);
    let rendered = format!("{}", count);
    rc.writer.write_all(rendered.into_bytes().as_ref())?;
    Ok(())
}

use super::BillType;

/// Takes a `T:ToJson` and a template path and does it's thing.
///
/// Returns path to created file, potenially in a `tempdir`.
// pub fn fill_template<E:ToJson>(document:E, template_file:&Path) -> PathBuf{
pub fn fill_template<E: ToJson, P:AsRef<Path>>(document: &E, bill_type:&BillType, template_path: P) -> Result<String> {
    let mut handlebars = Handlebars::new();

    handlebars.register_escape_fn(no_escape);
    handlebars.register_escape_fn(|data| data.replace("\n", r#"\newline "#));
    handlebars.register_helper("inc",   Box::new(inc_helper));
    handlebars.register_helper("count", Box::new(count_helper));

    handlebars.register_template_file("document", template_path).unwrap();

    let packed = match *bill_type {
        BillType::Offer => pack_data(document, false),
        BillType::Invoice => pack_data(document, true)
    };


    Ok(handlebars.render("document", &packed)
                 .map(|r| r.replace("<", "{")
                           .replace(">", "}"))?)
}

fn file_age(path:&Path) -> Result<time::Duration> {
    let metadata = fs::metadata(path)?;
    let accessed = metadata.accessed()?;
    Ok(accessed.elapsed()?)
}

fn output_template_path(template_name:&str) -> Result<PathBuf> {
    // construct_template_path(&template_name) {
    let template_ext  = ::CONFIG.get_str("extensions/output_template").expect("Faulty default config");
    let mut template_path = PathBuf::new();
    template_path.push(storage::get_storage_path());
    template_path.push(::CONFIG.get_str("dirs/templates").expect("Faulty config: dirs/templates does not contain a value"));
    template_path.push(template_name);
    template_path.set_extension(template_ext);
    // }

    // check stays here
    debug!("template file={:?} exists={}", template_path, template_path.exists());
    if template_path.exists() {
        Ok(template_path)
    } else {
        Err(format!("Template not found at {}", template_path.display()).into())
    }
}

/// Creates the latex files within each projects directory, either for Invoice or Offer.
#[cfg(feature="document_export")]
fn project_to_doc(project: &Project, config: &ExportConfig) -> Result<PathBuf> {

    let &ExportConfig {
        select: _,
        template_name: template_name,
        bill_type: bill_type,
        output: output_path,
        dry_run: dry_run,
        force: force,
        open: _
    } = config;

    // init_export_config()
    let output_ext    = ::CONFIG.get_str("extensions/output_file").expect("Faulty default config");
    let convert_ext   = ::CONFIG.get_str("convert/output_extension").expect("Faulty default config");
    let convert_tool  = ::CONFIG.get_str("convert/tool");
    let output_folder = ::CONFIG.get_str("output_path").and_then(util::get_valid_path).expect("Faulty config \"output_path\"");
    let trash_exts    = ::CONFIG.get("convert/trash_extensions") .expect("Faulty default config")
                                .as_vec().expect("Faulty default config")
                                .into_iter()
                                .map(|v|v.as_str()).collect::<Vec<_>>();

    let  template_path = output_template_path(template_name)?;

    // project_readyness(&project) {
    let ready_for_offer = project.is_ready_for_offer();
    let ready_for_invoice = project.is_ready_for_invoice();
    let project_file = project.file();

    // tiny little helper
    let to_local_file = |file:&Path, ext| {
        let mut _tmpfile = file.to_owned();
        _tmpfile.set_extension(ext);
        Path::new(_tmpfile.file_name().unwrap().into()).to_owned()
    };

    use BillType::*;
    let (dyn_bill_type, outfile_tex):
        (Option<BillType>, Option<PathBuf>) =
         match (bill_type, ready_for_offer, ready_for_invoice)
    {
        (Some(Offer),   Ok(_),  _     )  |
        (None,          Ok(_),  Err(_))  => (Some(Offer), Some(project.dir().join(project.offer_file_name(output_ext).expect("this should have been cought by ready_for_offer()")))),
        (Some(Invoice), _,      Ok(_))  |
        (None,          _,      Ok(_))  => (Some(Invoice), Some(project.dir().join(project.invoice_file_name(output_ext).expect("this should have been cought by ready_for_invoice()")))),
        (Some(Offer),   Err(e), _    )  => {error!("cannot create an offer, check out:{}",e);(None,None)},
        (Some(Invoice), _,      Err(e)) => {error!("cannot create an invoice, check out:{}",e);(None,None)},
        (_,         Err(e),     Err(_)) => {error!("Neither an Offer nor an Invoice can be created from this project\n please check out {}", e);(None,None)}
    };

    // }

    if let (Some(outfile), Some(dyn_bill)) = (outfile_tex, dyn_bill_type) {
        let filled = fill_template(project, &dyn_bill, &template_path)?;

        let pdffile = to_local_file(&outfile, convert_ext);
        let target = if let Some(output_path) = output_path {
            if output_path.is_dir() { // if dir, use my name and place in there
                trace!("output_path is dir");
                output_path.join(&pdffile)
            } else if output_path.parent().map(Path::exists).unwrap_or(false) {// if not dir, place at this path with this name
                trace!("output_path is file");
                output_path.to_owned()
            } else {
                warn!("{}", lformat!("Can't make sense of {}", output_path.display()));
                output_folder.join(&pdffile)
            }
        } else {
            output_folder.join(&pdffile)
        };
        debug!("output target will be {:?}", target);

        // ok, so apparently we can create a tex file, so lets do it
        if !force && target.exists() && file_age(&target)? < file_age(&project_file)? {
            // no wait, nothing has changed, so lets save ourselves the work
            info!("nothing to be done, {} is younger than {}
                         use --force if you don't agree",
                         //use --pdf to only rebuild the pdf",
                  target.display(),
                  project_file.display());
            Ok(target)
        } else {
            // \o/ we created a tex file

            if dry_run{
                warn!("Dry run! This does not produce any output:\n * {}\n * {}", outfile.display(), pdffile.display());
            } else {
                let outfile_path = project.write_to_file(&filled,&dyn_bill,output_ext)?;
                debug!("{} vs\n        {}", outfile.display(), outfile_path.display());
                util::pass_to_command(&convert_tool, &[&outfile_path]);
            }
            // clean up expected trash files
            for trash_ext in trash_exts.iter().filter_map(|x|*x){
                let trash_file = to_local_file(&outfile, trash_ext);
                if  trash_file.exists() {
                    fs::remove_file(&trash_file)?;
                    debug!("just deleted: {}", trash_file.display())
                } else {
                    debug!("I expected there to be a {}, but there wasn't any ?", trash_file.display())
                }
            }
            if pdffile.exists(){
                debug!("now there is be a {:?} -> {:?}", pdffile, target);
                fs::rename(&pdffile, &target)?;
                Ok(target)
            } else {
                bail!(error::ErrorKind::NoPdfCreated);
            }
        }
    } else {
        bail!(error::ErrorKind::NoPdfCreated);
    }
}

pub struct ExportConfig<'a> {
    pub select: StorageSelection,
    pub template_name: &'a str,
    pub bill_type: Option<BillType>,
    pub output: Option<&'a Path>,
    pub dry_run: bool,
    pub force: bool,
    pub open: bool,
}

impl<'a> Default for ExportConfig<'a> {
    fn default() -> Self {
        Self {
            select: StorageSelection::default(),
            template_name: "document",
            bill_type: None,
            output: None,
            dry_run: false,
            force: false,
            open: true
        }
    }
}

/// Creates the latex files within each projects directory, either for Invoice or Offer.
#[cfg(feature="document_export")]
pub fn projects_to_doc(config: &ExportConfig) {
    let storage = storage::setup::<Project>().unwrap();
    storage.with_selection(&config.select, |p| {
        project_to_doc(&p, &config)
            .map(|path| { if config.open {open::that(&path).unwrap();} } )
            .unwrap();
    });
}

