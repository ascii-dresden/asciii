use thiserror::Error;

use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("No PDF Created")]
    NoPdfCreated,

    #[error("Nothing to do")]
    NothingToDo,

    #[error("Template not found at {:?}", _0)]
    TemplateNotFoundAt(PathBuf),
}