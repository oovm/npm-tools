use std::path::Path;

use nifty_git::{collect_commits, discover_root, parse_github_remote_repo, resolve_range};

#[test]
fn discover_npm_tools_repo() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let git_root = discover_root(&root).expect("discover repo");
    assert!(git_root.join(".git").exists());
}

#[test]
fn parse_github_origin_url() {
    assert_eq!(parse_github_remote_repo("https://github.com/oovm/npm-tools.git"), Some("oovm/npm-tools".to_string()));
}

#[test]
fn resolve_range_by_version_when_tags_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let git_root = discover_root(&root).expect("discover repo");
    let tags = nifty_git::list_version_tags(&git_root).unwrap_or_default();
    if tags.len() < 2 {
        return;
    }
    let latest = tags.last().unwrap().trim_start_matches('v');
    let range = resolve_range(&git_root, Some(latest), None, None).expect("resolve range");
    assert_eq!(range.to_ref, format!("v{latest}"));
}

#[test]
fn collect_commits_returns_gitmoji_fields() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let git_root = discover_root(&root).expect("discover repo");
    let head = "HEAD";
    let commits = collect_commits(&git_root, None, head).expect("collect");
    assert!(!commits.is_empty());
    assert!(!commits[0].hash.is_empty());
    assert!(!commits[0].section.is_empty());
}
