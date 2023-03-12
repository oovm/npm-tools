use std::fs;
use std::path::{Path, PathBuf};

use nifty_formatter::{format_release_notes, ReleaseCommit};
use nifty_git::{collect_commits, resolve_range};
use nifty_types::{section_from_name, GithubAuthor};

use crate::{Result, UploadOptions};

pub fn resolve_notes(options: &UploadOptions, repo_root: &Path, tag: &str) -> Result<String> {
    if let Some(notes) = &options.notes {
        return Ok(notes.clone());
    }
    if let Some(path) = &options.notes_file {
        return fs::read_to_string(path).map_err(|err| format!("read notes file: {err}"));
    }
    if !options.generate_notes {
        return Ok(String::new());
    }
    generate_release_notes(repo_root, tag)
}

fn generate_release_notes(repo_root: &Path, tag: &str) -> Result<String> {
    let version = tag.trim_start_matches('v');
    let range = resolve_range(repo_root, Some(version), None, None).map_err(|err| err.to_string())?;
    let commits = collect_commits(
        repo_root,
        range.from_ref.as_deref(),
        range.to_ref.as_str(),
    )
    .map_err(|err| err.to_string())?;

    let author_map: std::collections::HashMap<String, GithubAuthor> = std::collections::HashMap::new();
    let release_commits: Vec<ReleaseCommit<'_>> = commits
        .iter()
        .map(|commit| ReleaseCommit {
            body: &commit.body,
            email: &commit.email,
            author: &commit.author,
            section: section_from_name(&commit.section).unwrap_or(nifty_types::Section::Other),
        })
        .collect();

    Ok(format_release_notes(&release_commits, &author_map))
}

pub fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Err(format!("upload directory not found: {}", dir.display()));
    }
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_map(|entry| entry.ok()) {
        if entry.file_type().is_file() {
            files.push(entry.into_path());
        }
    }
    if files.is_empty() {
        return Err(format!("no files found in {}", dir.display()));
    }
    files.sort();
    Ok(files)
}

pub fn asset_name(base_dir: &Path, file: &Path) -> String {
    file.strip_prefix(base_dir)
        .ok()
        .and_then(|relative| relative.to_str())
        .map(|path| path.replace('\\', "/"))
        .unwrap_or_else(|| {
            file.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("asset")
                .to_string()
        })
}
