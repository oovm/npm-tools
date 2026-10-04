//! `v*` tag 列举与 semver 排序。

use std::path::Path;

use semver::Version;

use crate::repo::{Result, open_repo, resolve_rev, short_oid};

/// Tag 名与短 hash。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagInfo {
    /// Annotated or lightweight tag name (e.g. `v1.0.0`).
    pub name: String,
    /// Short commit hash the tag points to.
    pub short_hash: String,
}

/// 列出 `v*` tag（semver 排序）。
pub fn list_version_tags(repo_root: &Path) -> Result<Vec<String>> {
    let tags = list_tag_infos(repo_root)?;
    Ok(tags.into_iter().map(|tag| tag.name).collect())
}

/// 列出 `v*` tag 及短 hash。
pub fn list_tag_infos(repo_root: &Path) -> Result<Vec<TagInfo>> {
    let repo = open_repo(repo_root)?;
    let mut names = Vec::new();
    for reference in repo.references().map_err(|err| err.to_string())?.all().map_err(|err| err.to_string())? {
        let reference = reference.map_err(|err| err.to_string())?;
        let name = reference.name().shorten().to_string();
        if name.starts_with('v') {
            names.push(name);
        }
    }
    names.sort_by(|left, right| compare_version_tags(left, right));
    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let oid = resolve_rev(&repo, &name)?;
        out.push(TagInfo { name, short_hash: short_oid(oid) });
    }
    Ok(out)
}

/// 格式化 `--tags` 输出。
pub fn format_tag_list(repo_root: &Path) -> Result<String> {
    let tags = list_tag_infos(repo_root)?;
    if tags.is_empty() {
        return Ok("(no v* tags)".to_string());
    }
    Ok(tags.into_iter().map(|tag| format!("{}\t{}", tag.name, tag.short_hash)).collect::<Vec<_>>().join("\n"))
}

fn compare_version_tags(left: &str, right: &str) -> std::cmp::Ordering {
    match (parse_tag_version(left), parse_tag_version(right)) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => left.cmp(right),
    }
}

fn parse_tag_version(tag: &str) -> Option<Version> {
    let trimmed = tag.trim_start_matches('v');
    Version::parse(trimmed).ok()
}

/// 去掉可选 `v` 前缀。
pub fn normalize_version(version: &str) -> String {
    version.trim_start_matches('v').to_string()
}

/// 返回 `vX.Y.Z` tag 名。
pub fn version_tag(version: &str) -> String {
    format!("v{}", normalize_version(version))
}

/// 查找目标版本的上一个 `v*` tag。
pub fn previous_version_tag(repo_root: &Path, version: &str) -> Result<Option<String>> {
    let target = version_tag(version);
    let tags = list_version_tags(repo_root)?;
    if let Some(index) = tags.iter().position(|tag| tag == &target) {
        if index > 0 {
            return Ok(Some(tags[index - 1].clone()));
        }
        return Ok(None);
    }
    let normalized = normalize_version(version);
    let parsed = Version::parse(&normalized).ok();
    if let Some(current) = parsed {
        if current.patch > 0 {
            let prev = format!("{}.{}.{}", current.major, current.minor, current.patch - 1);
            let tag = version_tag(&prev);
            if tags.iter().any(|existing| existing == &tag) {
                return Ok(Some(tag));
            }
        }
    }
    Ok(None)
}
