//! Fills export templates to create tex documents.
//!
//! Haven't decided on a templating engine yet, my own will probably not do.

use std::{io,fmt,time,fs};
use std::path::{Path, PathBuf};

use serde::ser::Serialize;

use open;
use handlebars::{RenderError, Handlebars, no_escape, Helper, RenderContext};

use util;
use project::{self, Project};
use project::export::ExportTarget;
use storage::error::StorageError;
use storage::{self, Storable, StorageSelection};

pub mod error;

use self::error::*;

#[cfg_attr(feature = "serialization", derive(Serialize))]
struct PackData<'a, T: 'a + Serialize> {
    document: &'a T,
    storage: storage::Paths,
    is_invoice:bool
}


fn pack_data<E: Serialize>(document: &E, is_invoice:bool) -> PackData<E> {
    PackData {
        document: document,
        storage: storage::setup::<Project>().unwrap().paths(),
        is_invoice:is_invoice
    }
}

fn inc_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> ::std::result::Result<(), RenderError> {
    // just for example, add error check for unwrap
    let param = h.param(0).expect("no param passed to inc_helper").value();
    debug!("inc_helper({:?})", param);
    let rendered = format!("{}", param.as_u64().expect("param can't be converted to u64") + 1);
    rc.writer.write_all(rendered.into_bytes().as_ref())?;
    Ok(())
}

fn count_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> ::std::result::Result<(), RenderError> {
    let count = h.param(0).unwrap().value().as_array().map_or(0, |a|a.len());
    //println!("count_helper{:?}", param);
    let rendered = format!("{}", count);
    rc.writer.write_all(rendered.into_bytes().as_ref())?;
    Ok(())
}

use super::BillType;

/// Takes a `T:Serialize` and a template path and does it's thing.
///
/// Returns path to created file, potenially in a `tempdir`.
// pub fn fill_template<E:Serialize>(document:E, template_file:&Path) -> PathBuf{
pub fn fill_template<E: Serialize, P:AsRef<Path>>(document: &E, bill_type:&BillType, template_path: P) -> Result<String> {
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
    let template_ext  = ::CONFIG.get_str("extensions/output_template");
    let mut template_path = PathBuf::new();
    template_path.push(storage::get_storage_path());
    template_path.push(::CONFIG.get_str("dirs/templates"));
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
    trace!("exporting a document: {:#?}", config);

    let &ExportConfig {
        select: _,
        template_name,
        bill_type,
        output: output_path,
        dry_run,
        force,
        open: _
    } = config;

    // init_export_config()
    let output_ext    = ::CONFIG.get_str("extensions/output_file");
    let convert_ext   = ::CONFIG.get_str("document_export/output_extension");
    let convert_tool  = ::CONFIG.get_str("document_export/convert_tool");
    let output_folder = util::get_valid_path(::CONFIG.get_str("output_path")).unwrap();
    let trash_exts    = ::CONFIG.get("document_export/trash_extensions")
                                .expect("Faulty default config")
                                .as_vec().expect("Faulty default config")
                                .into_iter()
                                .map(|v|v.as_str()).collect::<Vec<_>>();

    let  template_path = output_template_path(template_name)?;
    debug!("converting with {:?}", convert_tool);
    debug!("template {:?}", template_path);

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
        let exported_project: project::export::Complete = project.export();
        let filled = fill_template(&exported_project, &dyn_bill, &template_path)?;

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
            let mut outfile_path:PathBuf = PathBuf::new();
            if dry_run{
                warn!("Dry run! This does not produce any output:\n * {}\n * {}", outfile.display(), pdffile.display());
            } else {
                outfile_path = project.write_to_file(&filled,&dyn_bill,output_ext)?;
                debug!("{} vs\n        {}", outfile.display(), outfile_path.display());
                util::pass_to_command(&Some(convert_tool), &[&outfile_path]);
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

            outfile_path.set_extension("pdf");
            if pdffile.exists() || outfile_path.exists(){
                let file = match pdffile.exists() {
                    true => pdffile,
                    false => outfile_path,
                };
                debug!("now there is be a {:?} -> {:?}", file, target);
                fs::rename(&file, &target)?;
                Ok(target)
            } else {
                bail!(error::ErrorKind::NoPdfCreated);
            }
        }
    } else {
        bail!(error::ErrorKind::NoPdfCreated);
    }
}

#[derive(Debug)]
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
            template_name: ::CONFIG.get_str("document_export/default_template"),
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
pub fn projects_to_doc(config: &ExportConfig) -> Result<()> {
    let storage = storage::setup::<Project>()?;
    for p in storage.open_projects(&config.select)? {
        project_to_doc(&p, &config)
            .map(|path| { if config.open {open::that(&path).unwrap();} } )
            .unwrap()
    }
    Ok(())
}

