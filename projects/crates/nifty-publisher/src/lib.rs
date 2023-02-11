//! Publish npm workspace packages in dependency order.

mod graph;
mod manifest;
mod npm;
mod workspace;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub use graph::plan_publish_order;
pub use workspace::{NpmPackage, PackageManifest, find_workspace_root, list_workspace_packages};

pub type Result<T> = std::result::Result<T, String>;

/// Options for [`publish_workspace`].
#[derive(Debug, Clone, Default)]
pub struct PublishOptions {
    /// Directory used to locate the Nifty workspace.
    pub cwd: Option<PathBuf>,
    /// Run `npm publish --dry-run` for every package.
    pub dry_run: bool,
    /// npm dist-tag (for example `latest` or `next`).
    pub tag: Option<String>,
    /// npm `--access` flag for scoped packages.
    pub access: Option<String>,
}

/// Result of [`publish_workspace`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReport {
    pub root: PathBuf,
    pub order: Vec<String>,
    pub published: Vec<String>,
    pub skipped: Vec<String>,
}

/// Discover workspace npm packages, sort them with `petgraph`, and run `npm publish`.
pub fn publish_workspace(options: PublishOptions) -> Result<PublishReport> {
    let cwd = options
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let root = find_workspace_root(&cwd)?;
    let packages = list_workspace_packages(&root)?;
    let by_name = packages
        .iter()
        .map(|package| (package.name.clone(), package.clone()))
        .collect::<BTreeMap<_, _>>();
    let skipped = packages
        .iter()
        .filter(|package| package.private)
        .map(|package| package.name.clone())
        .collect::<Vec<_>>();
    let order = graph::sort_packages_for_publish(
        &packages.iter().filter(|package| !package.private).cloned().collect::<Vec<_>>(),
        &by_name,
    )?;

    let access = options.access.as_deref().or(Some("public"));
    let mut published = Vec::new();
    for name in &order {
        let package = by_name
            .get(name)
            .ok_or_else(|| format!("missing workspace package {name}"))?;
        publish_package(package, &by_name, options.dry_run, options.tag.as_deref(), access)?;
        published.push(name.clone());
    }

    Ok(PublishReport {
        root,
        order,
        published,
        skipped,
    })
}

fn publish_package(
    package: &NpmPackage,
    by_name: &BTreeMap<String, NpmPackage>,
    dry_run: bool,
    tag: Option<&str>,
    access: Option<&str>,
) -> Result<()> {
    let original = fs::read_to_string(&package.manifest_path).map_err(|err| err.to_string())?;
    let next = manifest::patch_manifest_for_publish(&original, package, by_name)?;

    let mut restored = false;
    let mut restore = || {
        if !restored {
            let _ = fs::write(&package.manifest_path, &original);
            restored = true;
        }
    };

    if next != original {
        fs::write(&package.manifest_path, &next).map_err(|err| err.to_string())?;
    }

    let result = npm::run_npm_publish(&package.dir, dry_run, tag, access);
    restore();
    result
}
