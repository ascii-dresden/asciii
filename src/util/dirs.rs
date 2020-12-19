use std::{env::var, path::PathBuf};

pub fn home_dir() -> Option<PathBuf> {
    var("HOME").map(PathBuf::from).ok()
}
