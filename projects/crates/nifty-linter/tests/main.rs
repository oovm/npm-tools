use nifty_linter::{
    LintOptions, RULE_CARGO_PACKAGE_SECTION, RULE_CARGO_README_CASE, RULE_CARGO_README_MISSING, RULE_GITMOJI_SUBJECT, run_lint,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nifty-linter-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("reset scratch dir");
    }
    fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn lint_at(dir: &Path) -> nifty_linter::LintReport {
    run_lint(LintOptions { cwd: Some(dir.to_path_buf()), ..Default::default() }).expect("lint")
}

fn write_manifest(dir: &Path, body: &str) {
    fs::write(dir.join("Cargo.toml"), body).expect("write manifest");
}

#[test]
fn lints_invalid_subject_via_cli_options() {
    let report = run_lint(LintOptions { subjects: vec!["Add feature".to_string()], ..Default::default() }).expect("lint");
    assert!(report.diagnostics.iter().any(|item| item.rule == RULE_GITMOJI_SUBJECT));
}

#[test]
fn accepts_valid_subject() {
    let report = run_lint(LintOptions { subjects: vec!["✨ Add feature".to_string()], ..Default::default() }).expect("lint");
    assert!(report.diagnostics.is_empty());
}

#[test]
fn readme_case_rule_catches_title_case_on_every_platform() {
    let dir = scratch("readme-title-case");
    write_manifest(&dir, "[package]\nname = \"probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n");
    fs::write(dir.join("Readme.md"), "# Probe\n").expect("write readme");

    let report = lint_at(&dir);
    assert!(report.diagnostics.iter().any(|item| item.rule == RULE_CARGO_README_CASE));
    assert!(!report.diagnostics.iter().any(|item| item.rule == RULE_CARGO_README_MISSING));
}

#[test]
fn lowercase_readme_is_accepted() {
    let dir = scratch("readme-lowercase");
    write_manifest(&dir, "[package]\nname = \"probe\"\nversion = \"0.0.0\"\nedition = \"2021\"\n");
    fs::write(dir.join("readme.md"), "# Probe\n").expect("write readme");

    let report = lint_at(&dir);
    assert!(!report.diagnostics.iter().any(|item| item.rule == RULE_CARGO_README_CASE));
    assert!(!report.diagnostics.iter().any(|item| item.rule == RULE_CARGO_README_MISSING));
}

#[test]
fn virtual_workspace_manifest_is_not_checked_for_package_section() {
    let dir = scratch("workspace-virtual");
    write_manifest(&dir, "[workspace]\nmembers = []\n");

    let report = lint_at(&dir);
    assert!(!report.diagnostics.iter().any(|item| item.rule == RULE_CARGO_PACKAGE_SECTION));
}
