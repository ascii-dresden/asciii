//! Fills export templates to create tex documents.
//!
//! Haven't decided on a templating engine yet, my own will probably not do.

use std::{time,fs};
use std::path::{Path, PathBuf};

use serde::ser::Serialize;
use failure::bail;

use open;
use handlebars::{Handlebars, no_escape, Helper, RenderContext, HelperDef, Context, Output, HelperResult};
use log::{info, debug, trace, error, warn};

use crate::util;
use crate::project::{self, Project, Exportable};
use crate::project::BillType::{self, Invoice, Offer};
use crate::project::export::ExportTarget;
use crate::storage::{self, Storable, StorageSelection};

pub mod error;

use self::error::*;

type Result<T> = ExportResult<T>;

#[cfg_attr(feature = "serialization", derive(Serialize))]
struct DocAndStorage<'a, T: Serialize> {
    document: &'a T,
    storage: Option<storage::Paths>,
    is_invoice: bool
}

impl<'a, T: 'a + Serialize> DocAndStorage<'a, T> {
    fn from(document: &T, bill_type: BillType) -> DocAndStorage<'_, T> {
        DocAndStorage {
            document,
            storage: storage::setup::<Project>().ok().map(|s| s.paths()),
            is_invoice: bill_type == Invoice
        }
    }
}

#[derive(Clone, Copy)]
struct IncHelper;

impl HelperDef for IncHelper {
    #[allow(clippy::extra_unused_lifetimes)]
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'_, '_>, _: &Handlebars, _: &Context, _: &mut RenderContext<'_>, out: &mut dyn Output) -> HelperResult {
        let param = h.param(0).unwrap().value();
        debug!("inc_helper({:?})", param);
        out.write(&format!("{}", param.as_u64().expect("param can't be converted to u64") + 1))?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct CountHelper;

impl HelperDef for CountHelper {
    #[allow(clippy::extra_unused_lifetimes)]
    fn call<'reg: 'rc, 'rc>(&self, h: &Helper<'_, '_>, _: &Handlebars, _: &Context, _: &mut RenderContext<'_>, out: &mut dyn Output) -> HelperResult {
        let count = h.param(0).unwrap().value().as_array().map_or(0, |a|a.len());
        out.write(&format!("{}", count))?;
        Ok(())
    }
}

/// Takes a `T: Serialize` and a template path and does it's thing.
///
/// Returns path to created file, potenially in a `tempdir`.
// pub fn fill_template<E:Serialize>(document:E, template_file:&Path) -> PathBuf{
pub fn fill_template<E, P>(document: &E, bill_type: BillType, template_path: P) -> Result<String>
    where E: Serialize, P:AsRef<Path>
{
    let mut handlebars = Handlebars::new();

    handlebars.register_escape_fn(no_escape);
    handlebars.register_escape_fn(|data| data.replace("\n", r#"\newline "#));

    handlebars.register_helper("inc",   Box::new(IncHelper));
    // handlebars.register_helper("count", Box::new(count_helper));

    handlebars.register_template_file("document", template_path).unwrap();

    Ok(handlebars.render("document", &DocAndStorage::from(document, bill_type))
                 .map(|r| r.replace("<", "{")
                           .replace(">", "}"))?)
}

fn file_age(path: &Path) -> Result<time::Duration> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    Ok(modified.elapsed()?)
}

fn output_template_path(template_name:&str) -> Result<PathBuf> {
    // construct_template_path(&template_name) {
    let template_ext  = crate::CONFIG.get_str("extensions/output_template");
    let mut template_path = PathBuf::new();
    template_path.push(storage::get_storage_path());
    template_path.push(crate::CONFIG.get_str("dirs/templates"));
    template_path.push(template_name);
    template_path.set_extension(template_ext);
    // }

    // check stays here
    debug!("template file={:?} exists={}", template_path, template_path.exists());
    if template_path.exists() {
        Ok(template_path)
    } else {
        Err(ExportError::TemplateNotFoundAt(template_path).into())
    }
}

/// Creates the latex files within each projects directory, either for Invoice or Offer.
#[cfg(feature="document_export")]
#[allow(clippy::cyclomatic_complexity)] // sorry
fn project_to_doc(project: &Project, config: &ExportConfig<'_>) -> Result<Option<PathBuf>> {
    trace!("exporting a document: {:#?}", config);

    let &ExportConfig {
        template_name,
        bill_type,
        output: output_path,
        dry_run,
        force,
        pdf_only,
        print_only,
        ..
    } = config;

    // init_export_config()
    let output_ext    = crate::CONFIG.get_str("extensions/output_file");
    let convert_ext   = crate::CONFIG.get_str("document_export/output_extension");
    let convert_tool  = crate::CONFIG.get_str("document_export/convert_tool");
    let output_folder = util::get_valid_path(crate::CONFIG.get_str("output_path")).unwrap();
    let trash_exts    = crate::CONFIG.get("document_export/trash_extensions")
                                .expect("Faulty default config")
                                .as_vec().expect("Faulty default config")
                                .iter()
                                .map(|v|v.as_str()).collect::<Vec<_>>();

    let  template_path = output_template_path(template_name)?;
    debug!("converting with {:?}", convert_tool);
    debug!("template {:?}", template_path);

    // project_readyness(&project) {
    let ready_for_offer = project.is_ready_for_offer();
    let ready_for_invoice = project.is_ready_for_invoice();
    let project_file = project.file();

    let (dyn_bill_type, outfile_tex):
        (Option<BillType>, Option<PathBuf>) =
         match (bill_type, ready_for_offer, ready_for_invoice)
    {
        (Some(Offer),   Ok(_),  _     ) |
        (None,          Ok(_),  Err(_)) => (Some(Offer), Some(project.dir()
                                                                    .join(project.offer_file_name(output_ext)
                                                                                 .expect("this should have been cought by ready_for_offer()")))),
        (Some(Invoice), _,      Ok(_) ) |
        (None,          _,      Ok(_) ) => (Some(Invoice), Some(project.dir()
                                                                       .join(project.invoice_file_name(output_ext)
                                                                                    .expect("this should have been cought by ready_for_invoice()")))),
        (Some(Offer),   Err(e), _     ) => {error!("cannot create an offer, check out:{}",e);(None,None)},
        (Some(Invoice), _,      Err(e)) => {error!("cannot create an invoice, check out:{}",e);(None,None)},
        (_,         Err(e),     Err(_)) => {error!("Neither an Offer nor an Invoice can be created from this project\n please check out {}", e);(None,None)}
    };

    // }

    if let (Some(tex_file), Some(dyn_bill)) = (outfile_tex, dyn_bill_type) {
        let exported_project: project::export::Complete = project.export();
        let filled = fill_template(&exported_project, dyn_bill, &template_path)?;

        let pdffile = util::to_local_file(&tex_file, convert_ext);

        let document_file = if let Some(output_path) = output_path {
            if output_path.is_dir() { // if dir, use my name and place in there
                trace!("output_path is dir");
                output_path.join(&pdffile)
            } else if output_path.parent().map(Path::exists).unwrap_or(false) {// if not dir, place at this path with this name
                trace!("output_path is file");
                output_path.to_owned()
            } else {
                println!("{}", lformat!("WARNING: Can't make sense of {}", output_path.display()));
                output_folder.join(&pdffile)
            }
        } else {
            output_folder.join(&pdffile)
        };

        debug!("document file will be {:?}", document_file);

        let defy = force || pdf_only;

        // ok, so apparently we can create a tex file, so lets do it
        if !defy && document_file.exists() && file_age(&tex_file)? < file_age(&project_file)? {
            // no wait, nothing has changed, so lets save ourselves the work
            use std::ffi::OsStr;
            info!("Nothing to do!\n{} is younger than {}\n\nuse --force if you don't agree\nuse --pdf to only render the pdf again",
                  tex_file.file_name().and_then(OsStr::to_str).unwrap(),
                  project_file.file_name().and_then(OsStr::to_str).unwrap()
                  );
            Ok(None)

        } else if dry_run { // just testing what is possible
            warn!("Dry run! This does not produce any output:\n * {}\n * {}", tex_file.display(), pdffile.display());
            Ok(None)

        } else if print_only { // for debugging or pipelining purposes
            debug!("only printing");
            println!("{}", filled);
            Ok(None)


        } else { // ok, we really have to work

            let mut outfile_path = if pdf_only {
                let (tex_age, project_age) = (file_age(&tex_file)?, file_age(&project_file)?);
                info!("recreating the pdf");
                debug!("{:?} -> {:?}", tex_file, document_file);
                debug!("{:?} -> {:?}", tex_age, project_age);
                if project_age < tex_age && !util::really(&lformat!("Project file is younger than pdf, continue anyway?")) {
                    return Ok(None)
                }
                project.full_file_path(dyn_bill, output_ext)?
            } else {
                let outfile_path = project.write_to_file(&filled, dyn_bill, output_ext)?;
                debug!("{} vs\n        {}", tex_file.display(), outfile_path.display());
                outfile_path
            };
            util::pass_to_command(Some(convert_tool), &[&outfile_path]);

            // clean up expected log and aux files etc
            for trash_ext in trash_exts.iter().filter_map(|x|*x) {
                let trash_file = util::to_local_file(&tex_file, trash_ext);
                if  trash_file.exists() {
                    fs::remove_file(&trash_file)?;
                    debug!("just deleted: {}", trash_file.display())
                } else {
                    debug!("I expected there to be a {}, but there wasn't any ?", trash_file.display())
                }
            }

            // now we move the created pdf
            outfile_path.set_extension("pdf");
            if pdffile.exists() || outfile_path.exists() {
                let file = if pdffile.exists() {
                    pdffile
                } else {
                    outfile_path
                };
                debug!("now there is be a {:?} -> {:?}", file, document_file);
                fs::rename(&file, &document_file)?;
            } else {
                bail!(ExportError::NoPdfCreated);
            }
            Ok(Some(document_file))
        }

    } else {
        bail!(ExportError::NoPdfCreated);
    }
}

#[derive(Debug)]
pub struct ExportConfig<'a> {
    pub select: StorageSelection,
    pub template_name: &'a str,
    pub bill_type: Option<BillType>,
    pub output: Option<&'a Path>,
    pub dry_run: bool,
    pub pdf_only: bool,
    pub force: bool,
    pub print_only: bool,
    pub open: bool,
}

impl<'a> Default for ExportConfig<'a> {
    fn default() -> Self {
        Self {
            select: StorageSelection::default(),
            template_name: crate::CONFIG.get_str("document_export/default_template"),
            bill_type: None,
            output: None,
            dry_run: false,
            pdf_only: false,
            force: false,
            print_only: false,
            open: true
        }
    }
}

/// Creates the latex files within each projects directory, either for Invoice or Offer.
#[cfg(feature="document_export")]
pub fn projects_to_doc(config: &ExportConfig<'_>) -> Result<()> {
    let storage = storage::setup::<Project>()?;
    for p in storage.open_projects(&config.select)? {
        if let Some(path) = project_to_doc(&p, &config)? {
            if config.open {
                open::that(&path).unwrap();
            }
        }
    }
    Ok(())
}

