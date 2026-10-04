use std::{fs, process::Command};

use nifty_publisher::{OtpOverrides, PublishOptions, plan_publish_order, publish_workspace};
use tempfile::tempdir;

fn write_package(root: &std::path::Path, dir_name: &str, name: &str, private: bool, deps_json: &str) {
    let dir = root.join("projects").join("packages").join(dir_name);
    fs::create_dir_all(&dir).expect("mkdir");
    let private_line = if private { "\n    \"private\": true," } else { "" };
    fs::write(
        dir.join("package.json"),
        format!(
            r#"{{
    "name": "{name}",
    "version": "0.0.0",{private_line}
    "dependencies": {deps_json}
}}"#
        ),
    )
    .expect("write package.json");
}

fn write_workspace_skeleton(root: &std::path::Path) {
    fs::create_dir_all(root.join("projects").join("crates")).expect("crates dir");
    fs::create_dir_all(root.join("projects").join("packages")).expect("packages dir");
}

#[test]
fn plans_dependency_order_from_manifests() {
    let root = tempdir().expect("tempdir");
    write_workspace_skeleton(root.path());
    write_package(root.path(), "platform", "@scope/platform", false, "{}");
    write_package(root.path(), "main", "@scope/main", false, r#"{ "@scope/platform": "file:../platform" }"#);

    let packages = nifty_publisher::list_workspace_packages(root.path()).expect("packages");
    let order = plan_publish_order(&packages).expect("order");
    assert_eq!(order, vec!["@scope/platform".to_string(), "@scope/main".to_string()]);
}

fn npm_available() -> bool {
    Command::new("npm").arg("--version").status().map(|status| status.success()).unwrap_or(false)
}

#[test]
fn publish_workspace_dry_run_restores_manifest() {
    if !npm_available() {
        return;
    }

    let root = tempdir().expect("tempdir");
    write_workspace_skeleton(root.path());
    write_package(root.path(), "platform", "@scope/platform", false, "{}");
    write_package(root.path(), "main", "@scope/main", false, r#"{ "@scope/platform": "file:../platform" }"#);

    let main_manifest = root.path().join("projects/packages/main/package.json");
    let before = fs::read_to_string(&main_manifest).expect("read manifest");

    let report = publish_workspace(PublishOptions {
        cwd: Some(root.path().to_path_buf()),
        dry_run: true,
        refresh: false,
        placeholder: false,
        tag: None,
        access: Some("public".to_string()),
        npm: None,
        otp: OtpOverrides::default(),
        trust: None,
        only: None,
        packages: None,
    })
    .expect("publish");

    assert_eq!(report.published.len(), 2);
    let after = fs::read_to_string(&main_manifest).expect("read manifest");
    assert_eq!(before, after);
}
