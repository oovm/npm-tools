use std::path::Path;

use nifty_git::collect_commits;

use crate::context::CommitInput;
use crate::Result;

pub fn load_commits(repo_root: &Path, from_ref: Option<&str>, to_ref: Option<&str>) -> Result<Vec<CommitInput>> {
    let to_ref = to_ref.unwrap_or("HEAD");
    let records = collect_commits(repo_root, from_ref, to_ref).map_err(|err| err.to_string())?;
    Ok(records
        .into_iter()
        .map(|record| CommitInput {
            hash: Some(record.hash),
            subject: record.subject,
            message: Some(record.message),
        })
        .collect())
}
