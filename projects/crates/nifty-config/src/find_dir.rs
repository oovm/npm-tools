use std::env::current_exe;

use super::*;

/// Look for a folder with that name, return error if no such folder.
///
/// # Arguments
///
/// * `start`: The starting directory for searching
/// * `name`: Target directory name
///
/// # Examples
///
/// ```
/// # use nifty_config::find_directory;
/// # use std::env::current_dir;
/// let dir = find_directory(&current_dir().unwrap(), "target").unwrap();
/// ```
pub fn find_directory(start: &Path, name: &str) -> Result<PathBuf> {
    let normed = ensure_file(start, name)?;
    let mut here = normed.as_path();
    while let Some(dir) = here.parent() {
        let path = here.join(name);
        if path.exists() {
            return match path.is_dir() {
                true => Ok(path),
                false => Err(Error::from_raw_os_error(22)),
            };
        }
        else {
            here = dir;
        }
    }
    Err(Error::from_raw_os_error(20))
}

/// Look for a folder with that name, if not found, create one in the current directory
///
/// # Arguments
///
/// * `start`: The starting directory for searching
/// * `name`: Target directory name
///
/// # Examples
///
/// ```
/// # use nifty_config::find_directory_or_create;
/// # use std::env::current_dir;
/// let dir = find_directory_or_create(&current_dir().unwrap(), "target").unwrap();
/// ```
pub fn find_directory_or_create(start: &Path, name: &str) -> Result<PathBuf> {
    match find_directory(start, name) {
        Ok(o) => return Ok(o),
        Err(_) => {}
    }
    let dir = ensure_directory(start)?.join(name);
    create_dir_all(&dir)?;
    Ok(dir)
}

/// Get the folder where the executable file is located
///
/// # Examples
///
/// ```
/// print!("{}", nifty_config::this_directory().unwrap().display());
/// ```
pub fn this_directory() -> Result<PathBuf> {
    match current_exe()?.canonicalize()?.parent() {
        Some(s) => Ok(s.to_path_buf()),
        None => Err(Error::from_raw_os_error(6)),
    }
}
