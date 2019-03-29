use failure::Fail;

use std::path::PathBuf;

#[derive(Fail, Debug)]
pub enum ExportError {
    #[fail(display = "No PDF Created")]
    NoPdfCreated,

    #[fail(display = "Nothing to do")]
    NothingToDo,

    #[fail(display = "Template not found at {:?}", _0)]
    TemplateNotFoundAt(PathBuf),
}