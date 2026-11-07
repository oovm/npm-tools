//! release 区间解析。

use std::path::Path;

use nifty_types::RefDomain;

use crate::{
    log::collect_commits,
    repo::{open_repo, verify_commit_ref},
    tags::{normalize_version, previous_version_tag, version_tag},
};

pub type Result<T> = std::result::Result<T, String>;

/// 解析后的 tag 区间。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeInfo {
    /// Version label for reference file names.
    pub version: String,
    /// Exclusive range start tag or ref.
    pub from_ref: Option<String>,
    /// Inclusive range end tag or ref.
    pub to_ref: String,
}

/// 解析 `--version` 或 `--from` / `--to` 区间。
pub fn resolve_range(repo_root: &Path, version: Option<&str>, from: Option<&str>, to: Option<&str>) -> Result<RangeInfo> {
    if let Some(version) = version {
        let tag = version_tag(version);
        let repo = open_repo(repo_root)?;
        verify_commit_ref(&repo, &tag, RefDomain::ReleaseTag)?;
        let prev = previous_version_tag(repo_root, version)?;
        return Ok(RangeInfo { version: normalize_version(version), from_ref: prev, to_ref: tag });
    }
    let Some(to) = to
    else {
        return Err("need version=X.Y.Z or to=REF (optional from=REF)".to_string());
    };
    let to_ref = to_ref_name(to);
    let repo = open_repo(repo_root)?;
    verify_commit_ref(&repo, &to_ref, RefDomain::Ref)?;
    if let Some(from_ref) = from {
        verify_commit_ref(&repo, from_ref, RefDomain::Ref)?;
    }
    let version_label = regex_version_label(&to_ref).unwrap_or_else(|| to_ref.trim_start_matches('v').to_string());
    Ok(RangeInfo { version: version_label, from_ref: from.map(str::to_string), to_ref })
}

/// Bare versions (`0.1.1`, `v0.1.2-beta.1`) become `vX.Y.Z`. Every other value is a git ref and is kept as written.
fn to_ref_name(to: &str) -> String {
    let candidate = to.strip_prefix('v').unwrap_or(to);
    if semver::Version::parse(candidate).is_ok() { version_tag(to) } else { to.to_string() }
}

fn regex_version_label(reference: &str) -> Option<String> {
    let trimmed = reference.trim_start_matches('v');
    semver::Version::parse(trimmed).ok().map(|_| trimmed.to_string())
}

/// 区间 + commit 列表。
pub fn collect_range_commits(
    repo_root: &Path,
    version: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<(RangeInfo, Vec<crate::log::CommitRecord>)> {
    let range = resolve_range(repo_root, version, from, to)?;
    let commits = collect_commits(repo_root, range.from_ref.as_deref(), &range.to_ref)?;
    Ok((range, commits))
}
