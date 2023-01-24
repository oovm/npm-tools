//! `origin` remote 解析。

use std::path::Path;

use crate::repo::{Result, open_repo};

/// 从 `origin` remote URL 解析 `owner/repo`。
pub fn detect_github_repo(repo_root: &Path) -> Result<Option<String>> {
    let repo = open_repo(repo_root)?;
    let config = repo.config_snapshot();
    let url = config.string("remote.origin.url").map(|value| value.to_string());
    Ok(url.as_deref().and_then(parse_github_remote_repo))
}

/// 从 remote URL 解析 `owner/repo`（不访问 git）。
pub fn parse_github_remote_repo(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        return normalize_repo_path(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("git@github.com:") {
        return normalize_repo_path(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("ssh://git@github.com/") {
        return normalize_repo_path(rest);
    }
    None
}

fn normalize_repo_path(path: &str) -> Option<String> {
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    if path.contains(':') || path.is_empty() || !path.contains('/') {
        return None;
    }
    Some(path.to_string())
}
