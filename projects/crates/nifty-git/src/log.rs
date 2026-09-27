//! commit 历史遍历（gix rev_walk，跳过 merge commit）。

use std::path::Path;

use gix::bstr::ByteSlice;
use gix::{Commit, ObjectId, Repository};

use nifty_types::{parse_subject, section_name};

use crate::repo::{Result, open_repo, resolve_rev};

/// 单条 commit（含 Nifty gitmoji 解析结果）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRecord {
    /// Full commit hash.
    pub hash: String,
    /// Author email.
    pub email: String,
    /// Author display name.
    pub author: String,
    /// Subject line.
    pub subject: String,
    /// Full commit message (subject + body).
    pub message: String,
    /// Subject without gitmoji.
    pub body: String,
    /// Leading gitmoji if any.
    pub gitmoji: Option<String>,
    /// Release section: features | fixes | breaking | other.
    pub section: String,
}

/// 收集 `from_ref..to_ref` 非 merge commit（`from_ref` 为 exclusive base）。
pub fn collect_commits(repo_root: &Path, from_ref: Option<&str>, to_ref: &str) -> Result<Vec<CommitRecord>> {
    let repo = open_repo(repo_root)?;
    let tip = resolve_rev(&repo, to_ref)?;
    let base = match from_ref {
        Some(reference) => Some(resolve_rev(&repo, reference)?),
        None => None,
    };
    let oids = commits_in_range(&repo, base, tip)?;
    let mut out = Vec::with_capacity(oids.len());
    for oid in oids {
        let commit = read_commit(&repo, oid)?;
        if commit.parent_ids().count() > 1 {
            continue;
        }
        out.push(to_record(oid, &commit));
    }
    Ok(out)
}

fn commits_in_range(repo: &Repository, exclusive_base: Option<ObjectId>, tip: ObjectId) -> Result<Vec<ObjectId>> {
    let mut walk = repo.rev_walk([tip]);
    if let Some(base) = exclusive_base {
        walk = walk.with_pruned([base]);
    }
    let mut ids = Vec::new();
    for info in walk.all().map_err(|err| err.to_string())? {
        ids.push(info.map_err(|err| err.to_string())?.id().detach());
    }
    ids.reverse();
    Ok(ids)
}

fn read_commit(repo: &Repository, oid: ObjectId) -> Result<Commit<'_>> {
    Ok(repo.find_object(oid).map_err(|err| err.to_string())?.into_commit())
}

fn to_record(oid: ObjectId, commit: &Commit<'_>) -> CommitRecord {
    let message = normalize_commit_message(commit.message_raw_sloppy());
    let subject = message
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let parsed = parse_subject(&subject);
    let email = commit.author().map(|author| author.email.to_string()).unwrap_or_default();
    let author = commit.author().map(|author| author.name.to_string()).unwrap_or_default();
    CommitRecord {
        hash: oid.to_string(),
        email,
        author,
        subject: subject.clone(),
        message,
        body: parsed.body,
        gitmoji: parsed.gitmoji,
        section: section_name(parsed.section).to_string(),
    }
}

/// Strip UTF-8 BOM that breaks gitmoji-leading-subject detection.
fn normalize_commit_message(raw: &gix::bstr::BStr) -> String {
    raw.to_str()
        .unwrap_or("")
        .trim_start_matches('\u{feff}')
        .to_string()
}
