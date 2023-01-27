//! Nifty git helpers built on **gix** (no `git` subprocess).
//!
//! Read-only repository operations for release changelogs and project tooling.

mod log;
mod range;
mod remote;
mod repo;
mod tags;

pub use log::{CommitRecord, collect_commits};
pub use range::{RangeInfo, collect_range_commits, resolve_range};
pub use remote::{detect_github_repo, parse_github_remote_repo};
pub use repo::{discover_root, open_repo, resolve_rev, short_oid, verify_commit_ref};
pub use tags::{TagInfo, format_tag_list, list_tag_infos, list_version_tags, normalize_version, previous_version_tag, version_tag};
