//! Differential test: `nifty.git.resolveRange` and `nifty change-logs` must accept the same range inputs.
//!
//! The CLI resolves ranges through `nifty-history`, the public API through this crate. Both are
//! separate implementations of one contract, so every accepted input is compared literally here.

use std::{fs, path::Path, process::Command};

use nifty_git::{list_version_tags, resolve_range as public_resolve_range};
use nifty_history::changelog::{list_version_tags as changelog_list_version_tags, resolve_range as changelog_resolve_range};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git").args(args).current_dir(root).output().expect("spawn git");
    assert!(output.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn commit(root: &Path, file: &str, message: &str) {
    fs::write(root.join(file), file).expect("write file");
    git(root, &["add", file]);
    git(root, &["commit", "-q", "-m", message]);
}

/// Repository with `v0.1.0`, `v0.1.1`, `v0.1.2-beta.1` tags, a `dev` branch, a remote-tracking
/// `origin/dev`, a `v9.9.9` branch that is not a tag, and two tags that point at a tree.
fn probe_repo() -> boost_fs::temp::TempFolder {
    let dir = boost_fs::temp::create_folder().expect("tempdir");
    let root = dir.path().to_path_buf();
    git(&root, &["init", "-q", "-b", "dev"]);
    git(&root, &["config", "user.name", "Probe"]);
    git(&root, &["config", "user.email", "probe@example.com"]);
    commit(&root, "a.txt", "🎂 Project initialized!");
    git(&root, &["tag", "v0.1.0"]);
    commit(&root, "b.txt", "✨ Add alpha");
    git(&root, &["tag", "v0.1.1"]);
    commit(&root, "c.txt", "🔖 Add beta");
    git(&root, &["tag", "v0.1.2-beta.1"]);
    commit(&root, "d.txt", "📝 Add gamma");
    git(&root, &["tag", "-a", "v0.1.3", "-m", "nifty range test"]);
    let tree = git(&root, &["rev-parse", "HEAD^{tree}"]);
    git(&root, &["tag", "treetag", &tree]);
    git(&root, &["tag", "-a", "atreetag", "-m", "nifty tree tag", &tree]);
    git(&root, &["update-ref", "refs/remotes/origin/dev", "HEAD"]);
    git(&root, &["branch", "v9.9.9", "HEAD"]);
    dir
}

type Triple = (String, Option<String>, String);

fn changelog(root: &Path, version: Option<&str>, from: Option<&str>, to: Option<&str>) -> Result<Triple, String> {
    changelog_resolve_range(root, version, from, to).map_err(|err| err.to_string())
}

fn public_api(root: &Path, version: Option<&str>, from: Option<&str>, to: Option<&str>) -> Result<Triple, String> {
    public_resolve_range(root, version, from, to).map(|info| (info.version, info.from_ref, info.to_ref))
}

#[test]
fn ref_inputs_resolve_identically() {
    let dir = probe_repo();
    let root = dir.path();
    let head = git(root, &["rev-parse", "HEAD"]);
    let short = &head[..8];
    let refs =
        ["0.1.1", "v0.1.1", "0.1.2-beta.1", "v0.1.2-beta.1", "0.1.3", "v0.1.3", "dev", "origin/dev", "HEAD", "HEAD~2", short];

    for to in refs {
        let changelog_range = changelog(root, None, Some("v0.1.0"), Some(to));
        let api_range = public_api(root, None, Some("v0.1.0"), Some(to));
        assert_eq!(changelog_range, api_range, "range resolvers drifted for --to {to}");
        assert!(changelog_range.is_ok(), "--to {to} should resolve");
    }

    let changelog_range = changelog(root, None, Some("v0.1.0"), Some("no-such-ref"));
    let api_range = public_api(root, None, Some("v0.1.0"), Some("no-such-ref"));
    assert!(changelog_range.is_err(), "an unknown ref must fail in change-logs");
    assert!(api_range.is_err(), "an unknown ref must fail in the public API");
}

#[test]
fn version_inputs_resolve_identically() {
    let dir = probe_repo();
    let root = dir.path();

    for version in ["0.1.1", "v0.1.1", "0.1.2-beta.1", "0.1.3"] {
        let changelog_range = changelog(root, Some(version), None, None);
        let api_range = public_api(root, Some(version), None, None);
        assert_eq!(changelog_range, api_range, "range resolvers drifted for --version {version}");
        assert!(changelog_range.is_ok(), "--version {version} should resolve");
    }

    let changelog_range = changelog(root, Some("9.9.10"), None, None);
    let api_range = public_api(root, Some("9.9.10"), None, None);
    assert!(changelog_range.is_err(), "a missing release tag must fail in change-logs");
    assert!(api_range.is_err(), "a missing release tag must fail in the public API");
}

#[test]
fn branch_names_are_not_release_tags() {
    let dir = probe_repo();
    let root = dir.path();
    let public_tags = list_version_tags(root).expect("public tag list");
    let changelog_tags = changelog_list_version_tags(root).expect("changelog tag list");
    assert_eq!(public_tags, ["v0.1.0", "v0.1.1", "v0.1.2-beta.1", "v0.1.3"].map(str::to_string).to_vec());
    assert_eq!(changelog_tags, public_tags);
}

#[test]
fn ref_errors_match_across_backends() {
    let dir = probe_repo();
    let root = dir.path();

    let cases = ["no-such-ref", "treetag", "atreetag"];
    for to in cases {
        let changelog_range = changelog(root, None, None, Some(to));
        let api_range = public_api(root, None, None, Some(to));
        assert_eq!(changelog_range, api_range, "ref errors drifted for --to {to}");
        assert!(changelog_range.is_err(), "--to {to} must fail");
    }

    assert_eq!(changelog(root, None, None, Some("no-such-ref")).unwrap_err(), "ref does not exist: \"no-such-ref\"");
    assert_eq!(changelog(root, None, None, Some("treetag")).unwrap_err(), "ref is not a commit: \"treetag\"");
    assert_eq!(changelog(root, None, None, Some("atreetag")).unwrap_err(), "ref is not a commit: \"atreetag\"");

    let changelog_from = changelog(root, None, Some("no-such-from"), Some("HEAD"));
    let api_from = public_api(root, None, Some("no-such-from"), Some("HEAD"));
    assert_eq!(changelog_from, api_from, "ref errors drifted for --from");
    assert_eq!(changelog_from.unwrap_err(), "ref does not exist: \"no-such-from\"");
}
