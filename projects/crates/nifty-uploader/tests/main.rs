use nifty_uploader::notes::{asset_name, collect_files};

#[test]
fn collects_files_recursively() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("a.txt"), b"a").expect("write");
    std::fs::create_dir_all(dir.path().join("nested")).expect("mkdir");
    std::fs::write(dir.path().join("nested/b.txt"), b"b").expect("write");

    let files = collect_files(dir.path()).expect("collect");
    assert_eq!(files.len(), 2);
    assert_eq!(asset_name(dir.path(), &files[0]), "a.txt");
}

#[test]
fn asset_name_preserves_relative_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let nested = dir.path().join("nested").join("file.bin");
    std::fs::create_dir_all(nested.parent().unwrap()).expect("mkdir");
    std::fs::write(&nested, b"x").expect("write");
    assert_eq!(asset_name(dir.path(), &nested), "nested/file.bin");
}
