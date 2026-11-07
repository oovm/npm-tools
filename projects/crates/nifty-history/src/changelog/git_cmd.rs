//! 只读 git 子进程与 tag 区间解析。

use std::{cmp::Ordering, path::Path, process::Command};

use crate::{
    changelog::gitmoji::{leading_gitmoji, section_for_gitmoji, strip_gitmoji},
    error::{Result, validation, validation_with_input},
};

/// 单条 commit 摘要（来自 `git log --format`）。
#[derive(Debug, Clone)]
pub struct CommitEntry {
    /// Full commit hash.
    pub hash: String,
    /// Author email.
    pub email: String,
    /// Author name.
    pub author: String,
    /// Subject line (first line of commit message).
    pub subject: String,
    /// Subject with gitmoji stripped.
    pub body: String,
    /// Leading gitmoji if present.
    pub gitmoji: Option<&'static str>,
    /// Release section derived from gitmoji.
    pub section: crate::changelog::gitmoji::Section,
}

/// 在仓库根目录执行 git 并返回 stdout（trim 尾部空白）。
pub fn git_output(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|err| validation(format!("spawn git: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if stderr.is_empty() { format!("git {} failed", args.join(" ")) } else { stderr };
        return Err(validation_with_input("git command failed", detail));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
}

/// 列出 `v*` tag（`version:refname` 排序）。
pub fn list_version_tags(repo_root: &Path) -> Result<Vec<String>> {
    let out = git_output(repo_root, &["tag", "-l", "v*", "--sort=version:refname"])?;
    Ok(if out.is_empty() { Vec::new() } else { out.lines().map(str::to_string).collect() })
}

/// 校验 ref 指向 commit。
pub fn verify_commit_ref(repo_root: &Path, reference: &str) -> Result<()> {
    git_output(repo_root, &["rev-parse", "--verify", &format!("{reference}^{{commit}}")])?;
    Ok(())
}

/// Tag names whose target commit belongs to `commits` (full 40-hex OIDs).
///
/// Rewriting history never moves tags, so callers warn when a tag in the rewritten range
/// still points at a pre-rewrite object.
pub fn tags_in_range(repo_root: &Path, commits: &[String]) -> Result<Vec<String>> {
    let out = git_output(repo_root, &["for-each-ref", "--format=%(refname:short) %(objectname) %(*objectname)", "refs/tags"])?;
    if out.is_empty() {
        return Ok(Vec::new());
    }
    let mut tags = Vec::new();
    for line in out.lines() {
        let mut fields = line.split_whitespace();
        let Some(name) = fields.next()
        else {
            continue;
        };
        let object = fields.next().unwrap_or_default();
        let peeled = fields.next().unwrap_or_default();
        let target = if peeled.is_empty() { object } else { peeled };
        if commits.iter().any(|oid| oid == target) {
            tags.push(name.to_string());
        }
    }
    Ok(tags)
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
    if !tags.iter().any(|tag| tag == &target) {
        return Ok(None);
    }
    Ok(select_previous_tag(&tags, &target))
}

/// Greatest release tag below `target` by semver precedence.
///
/// `git tag --sort=version:refname` puts `v0.1.2` before `v0.1.2-beta.1`, so plain positional
/// lookup would treat the stable release as the predecessor of its own pre-release.
fn select_previous_tag(tags: &[String], target: &str) -> Option<String> {
    let target_key = parse_version_key(target)?;
    tags.iter()
        .filter(|tag| tag.as_str() != target)
        .filter_map(|tag| parse_version_key(tag).map(|key| (tag, key)))
        .filter(|(_, key)| compare_version_keys(key, &target_key) == Ordering::Less)
        .max_by(|(_, left), (_, right)| compare_version_keys(left, right))
        .map(|(tag, _)| tag.clone())
}

struct VersionKey {
    core: (u64, u64, u64),
    pre: Option<Vec<String>>,
}

fn parse_version_key(tag: &str) -> Option<VersionKey> {
    static VERSION_TAG: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = VERSION_TAG.get_or_init(|| regex::Regex::new(r"^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$").unwrap());
    let caps = re.captures(tag)?;
    let core = (caps.get(1)?.as_str().parse().ok()?, caps.get(2)?.as_str().parse().ok()?, caps.get(3)?.as_str().parse().ok()?);
    let pre = caps.get(4).map(|value| value.as_str().split('.').map(str::to_string).collect());
    Some(VersionKey { core, pre })
}

fn compare_version_keys(left: &VersionKey, right: &VersionKey) -> Ordering {
    left.core.cmp(&right.core).then_with(|| compare_pre(left.pre.as_deref(), right.pre.as_deref()))
}

/// Semver precedence: a release sorts above any of its pre-releases.
fn compare_pre(left: Option<&[String]>, right: Option<&[String]>) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left), Some(right)) => {
            for (left_id, right_id) in left.iter().zip(right.iter()) {
                let order = compare_pre_id(left_id, right_id);
                if order != Ordering::Equal {
                    return order;
                }
            }
            left.len().cmp(&right.len())
        }
    }
}

fn compare_pre_id(left: &str, right: &str) -> Ordering {
    match (left.parse::<u64>(), right.parse::<u64>()) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        (Ok(_), Err(_)) => Ordering::Less,
        (Err(_), Ok(_)) => Ordering::Greater,
        (Err(_), Err(_)) => left.cmp(right),
    }
}

/// 列出 tag 与短 hash（`--tags` 子命令输出）。
pub fn format_tag_list(repo_root: &Path) -> Result<String> {
    let tags = list_version_tags(repo_root)?;
    if tags.is_empty() {
        return Ok("(no v* tags)".to_string());
    }
    let mut lines = Vec::new();
    for tag in tags {
        let short = git_output(repo_root, &["rev-parse", "--short", &tag])?;
        lines.push(format!("{tag}\t{short}"));
    }
    Ok(lines.join("\n"))
}

/// 解析 tag 区间内的非 merge commit。
pub fn collect_commits(repo_root: &Path, from_ref: Option<&str>, to_ref: &str) -> Result<Vec<CommitEntry>> {
    let range = match from_ref {
        Some(from) => format!("{from}..{to_ref}"),
        None => to_ref.to_string(),
    };
    let out = git_output(repo_root, &["log", &range, "--no-merges", "--format=%H%x1f%ae%x1f%an%x1f%s"])?;
    if out.is_empty() {
        return Ok(Vec::new());
    }
    let mut commits = Vec::new();
    for line in out.lines() {
        let mut parts = line.split('\x1f');
        let hash = parts.next().unwrap_or_default().to_string();
        let email = parts.next().unwrap_or_default().to_string();
        let author = parts.next().unwrap_or_default().to_string();
        let subject = parts.next().unwrap_or_default().to_string();
        let gitmoji = leading_gitmoji(&subject);
        let section = section_for_gitmoji(gitmoji);
        commits.push(CommitEntry {
            hash,
            email,
            author,
            subject: subject.clone(),
            body: strip_gitmoji(&subject),
            gitmoji,
            section,
        });
    }
    Ok(commits)
}

/// 解析 release 区间（`--version` 或 `--from` / `--to`）。
pub fn resolve_range(
    repo_root: &Path,
    version: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<(String, Option<String>, String)> {
    if let Some(version) = version {
        let tag = version_tag(version);
        verify_commit_ref(repo_root, &tag).map_err(|_| validation_with_input("release tag does not exist", tag.as_str()))?;
        let prev = previous_version_tag(repo_root, version)?;
        return Ok((normalize_version(version), prev, tag));
    }

    let Some(to) = to
    else {
        return Err(crate::error::validation("need --version=X.Y.Z or --to=REF (optional --from=REF)"));
    };

    let to_ref = to_ref_name(to);
    verify_commit_ref(repo_root, &to_ref).map_err(|_| validation_with_input("ref is not a commit", to_ref.as_str()))?;
    if let Some(from_ref) = from {
        verify_commit_ref(repo_root, from_ref).map_err(|_| validation_with_input("ref is not a commit", from_ref))?;
    }

    let version_label = regex_version_label(&to_ref).unwrap_or_else(|| to_ref.trim_start_matches('v').to_string());
    Ok((version_label, from.map(str::to_string), to_ref))
}

/// `--to` accepts both a bare version (`0.1.1` → `v0.1.1`, `0.1.2-beta.1` → `v0.1.2-beta.1`) and any git ref.
///
/// Branch names such as `dev` must stay untouched instead of being turned into `vdev`.
fn to_ref_name(to: &str) -> String {
    let candidate = to.strip_prefix('v').unwrap_or(to);
    if semver::Version::parse(candidate).is_ok() { version_tag(to) } else { to.to_string() }
}

fn regex_version_label(reference: &str) -> Option<String> {
    static VERSION: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = VERSION.get_or_init(|| regex::Regex::new(r"^v?(\d+\.\d+\.\d+)$").unwrap());
    re.captures(reference).and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

/// Display name of a rendered release: versions stay prefixed with `v`, plain refs stay as-is.
///
/// A branch range such as `v0.1.0..dev` is not a release version, so `vdev` must never appear
/// in the heading or in the suggested release-note file name.
pub fn release_name(label: &str) -> String {
    if label.starts_with(|c: char| c.is_ascii_digit()) { format!("v{label}") } else { label.to_string() }
}

#[cfg(test)]
mod tests {
    use super::{release_name, select_previous_tag, to_ref_name};

    #[test]
    fn bare_version_gains_a_v_prefix() {
        assert_eq!(to_ref_name("0.1.1"), "v0.1.1");
        assert_eq!(to_ref_name("0.1.2-beta.1"), "v0.1.2-beta.1");
        assert_eq!(to_ref_name("v0.1.2-beta.1"), "v0.1.2-beta.1");
    }

    #[test]
    fn version_tags_and_refs_are_kept() {
        assert_eq!(to_ref_name("v0.1.1"), "v0.1.1");
        assert_eq!(to_ref_name("dev"), "dev");
        assert_eq!(to_ref_name("origin/dev"), "origin/dev");
        assert_eq!(to_ref_name("HEAD~2"), "HEAD~2");
        assert_eq!(to_ref_name("4efe948"), "4efe948");
    }

    #[test]
    fn release_name_prefixes_versions_and_keeps_refs() {
        assert_eq!(release_name("0.1.1"), "v0.1.1");
        assert_eq!(release_name("0.1.2-beta.1"), "v0.1.2-beta.1");
        assert_eq!(release_name("dev"), "dev");
        assert_eq!(release_name("origin/dev"), "origin/dev");
    }

    #[test]
    fn previous_tag_never_picks_the_release_of_a_prerelease() {
        let tags: Vec<String> = ["v0.1.0", "v0.1.1", "v0.1.2", "v0.1.2-beta.1"].map(str::to_string).to_vec();
        assert_eq!(select_previous_tag(&tags, "v0.1.2-beta.1"), Some("v0.1.1".to_string()));
        assert_eq!(select_previous_tag(&tags, "v0.1.2"), Some("v0.1.2-beta.1".to_string()));
        assert_eq!(select_previous_tag(&tags, "v0.1.0"), None);
    }

    #[test]
    fn previous_tag_orders_prerelease_identifiers() {
        let tags: Vec<String> = ["v1.0.0-alpha.1", "v1.0.0-alpha.2", "v1.0.0-beta.1", "v1.0.0"].map(str::to_string).to_vec();
        assert_eq!(select_previous_tag(&tags, "v1.0.0"), Some("v1.0.0-beta.1".to_string()));
        assert_eq!(select_previous_tag(&tags, "v1.0.0-beta.1"), Some("v1.0.0-alpha.2".to_string()));
        assert_eq!(select_previous_tag(&tags, "v1.0.0-alpha.2"), Some("v1.0.0-alpha.1".to_string()));
    }

    #[test]
    fn previous_tag_skips_non_version_tags() {
        let tags: Vec<String> = ["v0.1.0", "vnightly"].map(str::to_string).to_vec();
        assert_eq!(select_previous_tag(&tags, "v0.1.0"), None);
    }
}

/// 从 `origin` remote URL 解析 `owner/repo`。
pub fn detect_github_repo(repo_root: &Path) -> Option<String> {
    let url = git_output(repo_root, &["remote", "get-url", "origin"]).ok()?;
    parse_github_repo_from_remote(&url)
}

/// 从 remote URL 解析 `owner/repo`（不访问 git）。
pub fn parse_github_remote_repo(url: &str) -> Option<String> {
    parse_github_repo_from_remote(url)
}

fn parse_github_repo_from_remote(url: &str) -> Option<String> {
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
