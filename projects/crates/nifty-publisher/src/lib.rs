//! Publish npm workspace packages in dependency order.

mod cache;
mod graph;
mod manifest;
mod npm;
mod otp;
mod trust;
mod trust_expect;
mod workspace;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use cache::{PlaceholderCache, CACHE_DIR_NAME, CACHE_FILE_NAME};
pub use graph::plan_publish_order;
pub use otp::{OtpAuth, OtpOverrides};
pub use trust::{TrustOptions, TrustReport, TRUST_ENV, TRUST_FILE, TRUST_REPO};
pub use workspace::{NpmPackage, PackageManifest, find_workspace_root, list_workspace_packages};

pub type Result<T> = std::result::Result<T, String>;

/// Options for [`publish_workspace`].
#[derive(Debug, Clone, Default)]
pub struct PublishOptions {
    pub cwd: Option<PathBuf>,
    pub dry_run: bool,
    pub refresh: bool,
    pub tag: Option<String>,
    pub access: Option<String>,
    pub npm: Option<PathBuf>,
    pub otp: OtpOverrides,
}

/// Result of [`publish_workspace`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReport {
    pub root: PathBuf,
    pub order: Vec<String>,
    pub published: Vec<String>,
    pub skipped: Vec<String>,
    pub skipped_versions: Vec<String>,
}

/// Discover workspace npm packages, sort them with `petgraph`, and run `npm publish`.
pub fn publish_workspace(options: PublishOptions) -> Result<PublishReport> {
    let cwd = options
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let root = find_workspace_root(&cwd)?;
    let auth = OtpAuth::load(&root, options.otp);
    let trust_expect = trust_expect::resolve_trust_expect(&root);
    let mut cache = cache::PlaceholderCache::load(&root, &trust_expect);
    let runner = npm::NpmRunner::new(options.npm.as_deref(), auth.clone());
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
    let mut skipped_versions = Vec::new();
    for name in &order {
        let package = by_name
            .get(name)
            .ok_or_else(|| format!("missing workspace package {name}"))?;
        let live = runner.view_version(name)?;
        let resolved = cache::resolve_published_version(&mut cache, name, live);
        if cache::should_skip_publish(
            &cache,
            name,
            &package.version,
            &resolved,
            options.refresh,
            options.dry_run,
        ) {
            println!("skip publish {name}@{} (already on registry)", package.version);
            skipped_versions.push(name.clone());
            continue;
        }
        if is_platform_package(name) && !has_staged_native_binary(package) {
            println!("skip publish {name} (no staged native binary)");
            continue;
        }
        println!("publishing {name}@{version}", name = name, version = package.version);
        let result = publish_package(
            package,
            &by_name,
            options.dry_run,
            options.tag.as_deref(),
            access,
            options.npm.as_deref(),
            &auth,
        );
        match result {
            Ok(()) => {
                if !options.dry_run {
                    cache.record_version(name, &package.version);
                    cache.save(&root)?;
                }
                published.push(name.clone());
            }
            Err(message) if already_published(&message) => {
                cache.record_version(name, &package.version);
                cache.save(&root)?;
                println!("skip publish {name}@{} (registry says already published)", package.version);
                skipped_versions.push(name.clone());
            }
            Err(message) => return Err(message),
        }
    }

    cache.save(&root)?;

    Ok(PublishReport {
        root,
        order,
        published,
        skipped,
        skipped_versions,
    })
}

fn is_platform_package(name: &str) -> bool {
    name.starts_with("@doki-land/nifty-")
}

fn has_staged_native_binary(package: &NpmPackage) -> bool {
    let lib_dir = package.dir.join("lib");
    if !lib_dir.is_dir() {
        return false;
    }
    fs::read_dir(&lib_dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .any(|entry| entry.path().extension().is_some_and(|ext| ext == "node"))
        })
        .unwrap_or(false)
}

fn already_published(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("cannot publish over")
        || lower.contains("previously published")
        || lower.contains("version already exists")
        || lower.contains("already been published")
}

pub fn trust_workspace(options: TrustOptions) -> Result<TrustReport> {
    trust::trust_workspace(options)
}

fn publish_package(
    package: &NpmPackage,
    by_name: &BTreeMap<String, NpmPackage>,
    dry_run: bool,
    tag: Option<&str>,
    access: Option<&str>,
    npm: Option<&Path>,
    auth: &OtpAuth,
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

    let result = npm::run_npm_publish(&package.dir, dry_run, tag, access, npm, auth);
    restore();
    result
}
