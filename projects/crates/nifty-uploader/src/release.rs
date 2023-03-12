use std::path::Path;

use crate::github::{
    api_create_release, api_release_by_tag, api_upload_asset, release_asset_already_exists, release_id, split_repo,
    GitHubClient,
};
use crate::notes::{asset_name, collect_files};
use crate::{Result, UploadOptions};

#[derive(Debug, Clone, Default)]
pub struct UploadReport {
    pub release_assets: Vec<String>,
    pub pages_branch: Option<String>,
}

pub fn upload_release_assets(
    repo: &str,
    token: &str,
    tag: &str,
    options: &UploadOptions,
    notes: &str,
) -> Result<Vec<String>> {
    let (owner, name) = split_repo(repo)?;
    let client = GitHubClient::new(token)?;
    let release = match api_release_by_tag(&client, &owner, &name, tag)? {
        Some(existing) => existing,
        None => {
            let release_name = options
                .release_name
                .clone()
                .unwrap_or_else(|| tag.to_string());
            api_create_release(&client, &owner, &name, tag, &release_name, notes, options.draft)?
        }
    };
    let release_id = release_id(&release)?;
    let files = collect_files(&options.dir)?;
    let mut uploaded = Vec::new();

    for file in files {
        let bytes = std::fs::read(&file).map_err(|err| format!("read {}: {err}", file.display()))?;
        let asset = asset_name(&options.dir, &file);
        match api_upload_asset(&client, &owner, &name, release_id, &asset, &bytes) {
            Ok(_) => {
                println!("release: uploaded {asset}");
                uploaded.push(asset);
            }
            Err(err) if release_asset_already_exists(&err) => {
                println!("release: skip {asset} (already uploaded)");
            }
            Err(err) => return Err(err),
        }
    }

    Ok(uploaded)
}
