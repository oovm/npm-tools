use std::path::PathBuf;

use nifty_config::{detect_project_layout, ProjectKind};
use nifty_updater::discover_package_dirs;

#[test]
fn discovers_hybrid_package_dirs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root");
    let layout = detect_project_layout(&root.join("projects/packages/nifty"));
    assert_eq!(layout.kind, ProjectKind::Hybrid);
    let packages = discover_package_dirs(&layout).expect("packages");
    assert!(!packages.is_empty());
    assert!(packages.iter().any(|dir| dir == &root));
}
