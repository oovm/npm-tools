use std::path::PathBuf;

use crate::{UploadOptions, UploadTarget};

/// Apply GitHub Actions environment defaults for faster CI uploads.
pub fn apply_github_action_defaults(options: &mut UploadOptions) -> crate::Result<()> {
    if options.token.is_none() {
        options.token = std::env::var("GITHUB_TOKEN").ok();
    }
    if options.repo.is_none() {
        if let Ok(repo) = std::env::var("GITHUB_REPOSITORY") {
            options.repo = Some(repo);
        }
    }
    if options.tag.is_none() {
        if let Ok(tag) = std::env::var("GITHUB_REF_NAME") {
            options.tag = Some(tag);
        }
    }
    if options.cwd.is_none() {
        if let Ok(cwd) = std::env::var("GITHUB_WORKSPACE") {
            options.cwd = Some(PathBuf::from(cwd));
        }
    }
    Ok(())
}

pub fn parse_target(pages: bool, release: bool, both: bool) -> crate::Result<UploadTarget> {
    if both {
        return Ok(UploadTarget::Both);
    }
    if pages && release {
        return Ok(UploadTarget::Both);
    }
    if pages {
        return Ok(UploadTarget::Pages);
    }
    if release {
        return Ok(UploadTarget::Release);
    }
    Ok(UploadTarget::Release)
}
