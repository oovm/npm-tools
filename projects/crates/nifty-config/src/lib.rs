#![warn(missing_docs)]
#![doc = include_str!("../Readme.md")]

use std::{
    fs::create_dir_all,
    io::{Error, Result},
    path::{Path, PathBuf},
};

pub use find_dir::{find_directory, find_directory_or_create, this_directory};
pub use find_file::{CONFIG_FILE_NAMES, find_config_file};
pub use project::{ProjectKind, ProjectLayout, detect_project_layout, find_cargo_manifest, find_package_manifest};

mod find_dir;
mod find_file;
mod project;

/// Ensure path is dir
pub fn ensure_directory(path: &Path) -> Result<PathBuf> {
    if path.is_dir() {
        path.canonicalize()
    }
    else {
        match path.parent() {
            Some(s) => s.canonicalize(),
            None => Err(Error::from_raw_os_error(10006)),
        }
    }
}

/// Ensure path is file
pub fn ensure_file(path: &Path, name: &str) -> Result<PathBuf> {
    if path.is_file() { path.canonicalize() } else { Ok(path.canonicalize()?.join(name)) }
}
