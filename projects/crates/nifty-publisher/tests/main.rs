use std::{fs, process::Command};

use boost_fs::temp::create_folder;
use nifty_publisher::{
    OtpOverrides, PlaceholderCache, PublishOptions, TRUST_ENV, TRUST_FILE, TRUST_REPO, TrustExpect, plan_publish_order,
    publish_workspace, trust_expect::TrustExpectInput,
};

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
    let root = create_folder().expect("tempdir");
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

fn node_available() -> bool {
    Command::new("node").arg("--version").output().map(|output| output.status.success()).unwrap_or(false)
}

/// Write a fake npm executable that reports registry failures per package name.
///
/// `fail-package.txt` and `staged-package.txt` name the packages that must fail
/// with an auth error and a staged-pending conflict. Every other package succeeds
/// and writes `published.marker` into its directory.
fn write_fake_npm(root: &std::path::Path) -> std::path::PathBuf {
    let bin = root.join("fake-npm");
    fs::create_dir_all(&bin).expect("fake npm dir");
    fs::write(
        bin.join("fake-npm.cjs"),
        r#"const fs = require("fs");
const path = require("path");
const args = process.argv.slice(2);
if (args[0] === "view") {
    process.exit(1);
}
const dir = process.cwd();
const manifest = JSON.parse(fs.readFileSync(path.join(dir, "package.json"), "utf8"));
const configured = (file) => fs.readFileSync(path.join(__dirname, file), "utf8").trim();
if (manifest.name === configured("fail-package.txt")) {
    console.error("npm error code E401");
    console.error("npm error 401 Unauthorized - PUT https://registry.npmjs.org/" + manifest.name);
    process.exit(1);
}
if (manifest.name === configured("staged-package.txt")) {
    console.error("npm error code E409");
    console.error(
        "npm error 409 Conflict - PUT https://registry.npmjs.org/" + manifest.name +
            " - Cannot publish over previously staged version " + manifest.version + ".",
    );
    process.exit(1);
}
fs.writeFileSync(path.join(dir, "published.marker"), "ok");
process.exit(0);
"#,
    )
    .expect("write fake npm script");

    #[cfg(windows)]
    let executable = {
        let path = bin.join("npm.cmd");
        fs::write(&path, "@echo off\r\nnode \"%~dp0fake-npm.cjs\" %*\r\n").expect("write npm.cmd");
        path
    };
    #[cfg(not(windows))]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        let path = bin.join("npm");
        fs::write(&path, "#!/bin/sh\nexec node \"$(dirname \"$0\")/fake-npm.cjs\" \"$@\"\n").expect("write npm");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod npm");
        path
    };
    executable
}

#[test]
fn publish_continues_after_failure_and_keeps_staged_out_of_cache() {
    if !node_available() {
        return;
    }

    let root = create_folder().expect("tempdir");
    write_workspace_skeleton(root.path());
    write_package(root.path(), "one", "@scope/one", false, "{}");
    write_package(root.path(), "two", "@scope/two", false, "{}");
    write_package(root.path(), "three", "@scope/three", false, "{}");

    let packages = nifty_publisher::list_workspace_packages(root.path()).expect("packages");
    let order = plan_publish_order(&packages).expect("order");
    assert_eq!(order.len(), 3);
    let failure_name = order[0].clone();
    let staged_name = order[1].clone();
    let success_name = order[2].clone();

    let npm = write_fake_npm(root.path());
    fs::write(npm.parent().expect("fake npm dir").join("fail-package.txt"), &failure_name).expect("fail package");
    fs::write(npm.parent().expect("fake npm dir").join("staged-package.txt"), &staged_name).expect("staged package");

    let trust = TrustExpectInput {
        repo: Some(TRUST_REPO.to_string()),
        file: Some(TRUST_FILE.to_string()),
        env: Some(TRUST_ENV.to_string()),
    };
    let error = publish_workspace(PublishOptions {
        cwd: Some(root.path().to_path_buf()),
        dry_run: false,
        refresh: false,
        placeholder: false,
        tag: None,
        access: Some("public".to_string()),
        npm: Some(npm),
        otp: OtpOverrides::default(),
        trust: Some(trust),
        only: None,
        packages: None,
        version: None,
    })
    .expect_err("auth failure must fail the batch");

    assert!(error.contains(&failure_name), "aggregated error must name the failed package: {error}");
    assert!(error.contains("publish failed for 1 package(s)"), "unexpected error: {error}");

    // Both packages after the failure were still attempted.
    let success_dir = root.path().join("projects/packages").join(success_name.trim_start_matches("@scope/"));
    assert!(success_dir.join("published.marker").is_file(), "publish must continue after an independent package failure");

    let expect = TrustExpect { repo: TRUST_REPO.to_string(), file: TRUST_FILE.to_string(), env: TRUST_ENV.to_string() };
    let cache = PlaceholderCache::load(root.path(), &expect);
    assert_eq!(cache.cached_version(&success_name), Some("0.0.0"));
    assert_eq!(cache.cached_version(&failure_name), None);
    assert_eq!(cache.cached_version(&staged_name), None, "staged must not be cached as live");
}

#[test]
fn publish_workspace_dry_run_restores_manifest() {
    if !npm_available() {
        return;
    }

    let root = create_folder().expect("tempdir");
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
        version: None,
    })
    .expect("publish");

    assert_eq!(report.published.len(), 2);
    let after = fs::read_to_string(&main_manifest).expect("read manifest");
    assert_eq!(before, after);
}
