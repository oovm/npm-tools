//! Publish artifacts to GitHub Pages and GitHub Releases.

mod action;
mod github;
mod notes;
mod pages;
mod release;

use std::path::PathBuf;

pub use action::{apply_github_action_defaults, parse_target};
pub use release::UploadReport;

pub type Result<T> = std::result::Result<T, String>;

/// Upload target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UploadTarget {
    #[default]
    Release,
    Pages,
    Both,
}

/// Upload options for [`run_upload`].
#[derive(Debug, Clone)]
pub struct UploadOptions {
    pub cwd: Option<PathBuf>,
    pub repo: Option<String>,
    pub token: Option<String>,
    pub tag: Option<String>,
    pub release_name: Option<String>,
    pub notes: Option<String>,
    pub notes_file: Option<PathBuf>,
    pub dir: PathBuf,
    pub target: UploadTarget,
    pub github_action: bool,
    pub draft: bool,
    pub generate_notes: bool,
}

impl Default for UploadOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            repo: None,
            token: None,
            tag: None,
            release_name: None,
            notes: None,
            notes_file: None,
            dir: PathBuf::from("dist"),
            target: UploadTarget::Release,
            github_action: false,
            draft: false,
            generate_notes: true,
        }
    }
}

/// Upload to GitHub Pages and/or Release.
pub fn run_upload(mut options: UploadOptions) -> Result<UploadReport> {
    if options.github_action {
        apply_github_action_defaults(&mut options)?;
    }

    let token = options
        .token
        .clone()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .ok_or_else(|| "GitHub token is required (set --token or GITHUB_TOKEN)".to_string())?;

    let cwd = options
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let layout = nifty_config::detect_project_layout(&cwd);
    let repo_root = layout.root.clone();

    let repo = options
        .repo
        .clone()
        .or_else(|| nifty_git::detect_github_repo(&repo_root).ok().flatten())
        .ok_or_else(|| "could not detect GitHub repo (use --repo owner/name)".to_string())?;

    let tag = options
        .tag
        .clone()
        .or_else(|| std::env::var("GITHUB_REF_NAME").ok())
        .ok_or_else(|| "release tag is required (use --tag or push a git tag)".to_string())?;

    let notes = notes::resolve_notes(&options, &repo_root, &tag)?;
    let mut report = UploadReport::default();

    match options.target {
        UploadTarget::Release | UploadTarget::Both => {
            report.release_assets =
                release::upload_release_assets(&repo, &token, &tag, &options, &notes)?;
        }
        UploadTarget::Pages => {}
    }

    match options.target {
        UploadTarget::Pages | UploadTarget::Both => {
            pages::deploy_github_pages(&repo, &token, &options.dir)?;
            report.pages_branch = Some("gh-pages".to_string());
        }
        UploadTarget::Release => {}
    }

    Ok(report)
}
