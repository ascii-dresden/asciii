#![cfg(feature="document_export")]
//! Fills export templates to create tex documents.
//!
//! Haven't decided on a templating engine yet, my own will probably not do.

#![allow(unused_variables, dead_code)]

use std::path::{Path,PathBuf};

/// A type that implements this can be used to fill an export template.
pub trait Exportable{
}

/// Takes an exportable and a template path and does it's thing.
///
/// Returns path to created file, potenially in a `tempdir`.
pub fn fill_template<E:Exportable>(document:E, template_file:&Path) -> PathBuf{
    unimplemented!()
}
