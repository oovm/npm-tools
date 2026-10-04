use std::{env::current_dir, fs::File, io::Write};

use nifty_config::{CONFIG_FILE_NAMES, find_config_file, find_directory, find_directory_or_create};

#[test]
fn config_file_names_include_ts_and_js() {
    assert!(CONFIG_FILE_NAMES.contains(&"nifty.config.ts"));
    assert!(CONFIG_FILE_NAMES.contains(&"nifty.config.js"));
}

#[test]
fn find_config_file_in_tree() -> std::io::Result<()> {
    let dir = tempfile::tempdir()?;
    let nested = dir.path().join("a").join("b");
    std::fs::create_dir_all(&nested)?;
    let config = dir.path().join("nifty.config.js");
    File::create(&config)?.write_all(b"export default {}")?;

    assert_eq!(find_config_file(&nested), Some(config.canonicalize()?));
    Ok(())
}

#[test]
fn find_directory_works() -> std::io::Result<()> {
    let dir = current_dir()?;
    find_directory(&dir, "target")?;
    find_directory_or_create(&dir, "target")?;
    Ok(())
}
