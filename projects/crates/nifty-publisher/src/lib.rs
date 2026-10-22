//! Publish npm workspace packages in dependency order.

mod cache;
mod graph;
mod manifest;
mod npm;
mod npmrc;
mod otp;
mod registry_trust;
mod trust;
pub mod trust_expect;
mod workspace;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub use cache::{CACHE_DIR_NAME, CACHE_FILE_NAME, PlaceholderCache};
pub use graph::plan_publish_order;
pub use otp::{OtpAuth, OtpOverrides};
pub use trust::{TRUST_ENV, TRUST_FILE, TRUST_REPO, TrustOptions, TrustReport};
pub use workspace::{NpmPackage, PackageManifest, find_workspace_root, list_workspace_packages, registry_name};

pub type Result<T> = std::result::Result<T, String>;

const PLACEHOLDER_VERSION: &str = "0.0.0";

/// Options for [`publish_workspace`].
#[derive(Debug, Clone, Default)]
pub struct PublishOptions {
    pub cwd: Option<PathBuf>,
    pub dry_run: bool,
    pub refresh: bool,
    /// Publish never-published workspace packages as `0.0.0` stubs (claim name / OIDC prep).
    pub placeholder: bool,
    pub tag: Option<String>,
    pub access: Option<String>,
    pub npm: Option<PathBuf>,
    pub otp: OtpOverrides,
    /// Publish a single package name (`--package` when passed once).
    pub only: Option<String>,
    /// Publish an explicit subset (multiple `--package` flags).
    pub packages: Option<Vec<String>>,
    /// Override the published version for every package (tag-driven releases).
    pub version: Option<String>,
    pub trust: Option<trust_expect::TrustExpectInput>,
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
    let cwd = options.cwd.clone().unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let root = find_workspace_root(&cwd)?;
    let publish_targets = resolve_publish_targets(&options)?;
    let auth = OtpAuth::load(&root, options.otp);
    let trust_expect = trust_expect::resolve_trust_expect(&root, options.trust.as_ref());
    let mut cache = cache::PlaceholderCache::load(&root, &trust_expect);
    let runner = npm::NpmRunner::new(options.npm.as_deref(), auth.clone());
    let packages = list_workspace_packages(&root)?;
    let by_name = packages.iter().map(|package| (package.name.clone(), package.clone())).collect::<BTreeMap<_, _>>();
    let (candidates, skipped) = resolve_candidate_packages(&packages, &by_name, &publish_targets)?;
    let order = graph::sort_packages_for_publish(&candidates, &by_name)?;

    let access = options.access.as_deref().or(Some("public"));
    let mut published = Vec::new();
    let mut skipped_versions = Vec::new();
    let mut failures: Vec<(String, String)> = Vec::new();

    // Placeholder: only packages with no registry version yet. Force publish as 0.0.0.
    let mut placeholder_names = BTreeSet::new();
    if options.placeholder {
        for name in &order {
            let package = by_name.get(name).ok_or_else(|| format!("missing workspace package {name}"))?;
            let registry = registry_name(package);
            let live = runner.view_version(registry)?;
            if live.is_some() {
                println!("skip placeholder {registry} (already on registry)");
                skipped_versions.push(name.clone());
                continue;
            }
            placeholder_names.insert(name.clone());
        }
    }

    for name in &order {
        if options.placeholder && !placeholder_names.contains(name) {
            continue;
        }
        let package = by_name.get(name).ok_or_else(|| format!("missing workspace package {name}"))?;
        let registry = registry_name(package);
        let publish_version = if options.placeholder {
            PLACEHOLDER_VERSION
        }
        else {
            options.version.as_deref().unwrap_or(package.version.as_str())
        };
        if !options.placeholder {
            let live = runner.view_version(registry)?;
            let resolved = cache::resolve_published_version(&mut cache, name, live);
            if cache::should_skip_publish(&cache, name, publish_version, &resolved, options.refresh, options.dry_run) {
                println!("skip publish {registry}@{publish_version} (already on registry)");
                skipped_versions.push(name.clone());
                continue;
            }
        }
        if is_native_binary_package(package) && !has_staged_native_binary(package) && !options.placeholder {
            println!("skip publish {name} (no staged native binary)");
            continue;
        }
        println!("publishing {registry}@{publish_version}");
        let result = publish_package(
            package,
            &by_name,
            options.dry_run,
            options.tag.as_deref(),
            access,
            options.npm.as_deref(),
            &auth,
            if options.placeholder { Some(PLACEHOLDER_VERSION) } else { Some(publish_version) },
            if options.placeholder { Some(&placeholder_names) } else { None },
        );
        match result {
            Ok(()) => {
                if !options.dry_run {
                    cache.record_version(name, publish_version);
                    cache.save(&root)?;
                }
                published.push(name.clone());
            }
            Err(message) => {
                match classify_publish_error(&message) {
                    PublishErrorKind::AlreadyLive => {
                        cache.record_version(name, publish_version);
                        cache.save(&root)?;
                        println!("skip publish {name}@{publish_version} (registry says already live)");
                        skipped_versions.push(name.clone());
                    }
                    PublishErrorKind::StagedPending => {
                        println!(
                            "skip publish {name}@{publish_version} (registry has a staged pending version — not yet live)"
                        );
                        // Do NOT record into version cache: staged is not live.
                        skipped_versions.push(name.clone());
                    }
                    PublishErrorKind::Other => {
                        println!("publish failed for {name}: {message}");
                        if let Err(aggregated) = aggregate_failure(&mut failures, name, &message) {
                            return Err(aggregated);
                        }
                    }
                }
            }
        }
    }

    cache.save(&root)?;

    if !failures.is_empty() {
        let joined = failures.iter().map(|(name, msg)| format!("  - {name}: {msg}")).collect::<Vec<_>>().join("\n");
        return Err(format!("publish failed for {} package(s):\n{joined}", failures.len()));
    }

    Ok(PublishReport { root, order, published, skipped, skipped_versions })
}

fn is_native_binary_package(package: &NpmPackage) -> bool {
    package.manifest.os.as_ref().is_some_and(|os| !os.is_empty())
        && package.manifest.cpu.as_ref().is_some_and(|cpu| !cpu.is_empty())
}

fn has_staged_native_binary(package: &NpmPackage) -> bool {
    let has_node = |dir: &Path| {
        fs::read_dir(dir)
            .map(|entries| {
                entries.filter_map(|entry| entry.ok()).any(|entry| entry.path().extension().is_some_and(|ext| ext == "node"))
            })
            .unwrap_or(false)
    };
    // Two shipped layouts: `lib/<addon>.node` (nifty) and `<addon>.node` at the package root (vmz).
    has_node(&package.dir.join("lib")) || has_node(&package.dir)
}

/// Classification of an `npm publish` failure message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PublishErrorKind {
    /// The version is already on the live registry — safe to skip and cache.
    AlreadyLive,
    /// The registry has a staged pending version that has not yet been promoted to live.
    /// Must NOT be cached as live.
    StagedPending,
    /// Any other publish failure (auth, network, dep conflict, etc.).
    Other,
}

/// Classify an `npm publish` error message without mutating any state.
///
/// Order matters: staged-pending must be checked before the generic
/// "cannot publish over" live-match, because npm's staged error begins with
/// the same prefix ("Cannot publish over previously staged version...")
/// as the live error ("Cannot publish over previously published version...").
fn classify_publish_error(message: &str) -> PublishErrorKind {
    let lower = message.to_ascii_lowercase();
    if lower.contains("previously staged") || lower.contains("staged pending") {
        return PublishErrorKind::StagedPending;
    }
    if lower.contains("cannot publish over")
        || lower.contains("previously published")
        || lower.contains("version already exists")
        || lower.contains("already been published")
    {
        return PublishErrorKind::AlreadyLive;
    }
    PublishErrorKind::Other
}

fn aggregate_failure(failures: &mut Vec<(String, String)>, name: &str, message: &str) -> std::result::Result<(), String> {
    failures.push((name.to_string(), message.to_string()));
    Ok(())
}

pub fn trust_workspace(options: TrustOptions) -> Result<TrustReport> {
    trust::trust_workspace(options)
}

/// Resolve which workspace packages participate in this publish.
///
/// Default: all publishable packages (`private: false` and not blocked by a required private dep).
/// With `--package` / `publish.packages`, narrow to that subset; `private` is never overridden.
fn resolve_candidate_packages(
    packages: &[NpmPackage],
    by_name: &BTreeMap<String, NpmPackage>,
    targets: &[String],
) -> Result<(Vec<NpmPackage>, Vec<String>)> {
    use crate::workspace::{find_package_by_target, unpublishable_package_names};

    let blocked = unpublishable_package_names(packages, by_name);

    if targets.is_empty() {
        let skipped = packages
            .iter()
            .filter(|package| blocked.contains(&package.name))
            .map(|package| package.name.clone())
            .collect::<Vec<_>>();
        let candidates = packages.iter().filter(|package| !blocked.contains(&package.name)).cloned().collect::<Vec<_>>();
        return Ok((candidates, skipped));
    }

    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    for target in targets {
        let Some(package) = find_package_by_target(by_name, target)
        else {
            return Err(format!("workspace package not found: {target}"));
        };
        if blocked.contains(&package.name) {
            println!(
                "skip publish target {target} (private workspace package or depends on one — remove `private` to publish)"
            );
            continue;
        }
        if seen.insert(package.name.clone()) {
            candidates.push(package.clone());
        }
    }
    let candidate_names = candidates.iter().map(|package| package.name.clone()).collect::<BTreeSet<_>>();
    let skipped = packages
        .iter()
        .filter(|package| blocked.contains(&package.name) && !candidate_names.contains(&package.name))
        .map(|package| package.name.clone())
        .collect::<Vec<_>>();
    Ok((candidates, skipped))
}

fn resolve_publish_targets(options: &PublishOptions) -> Result<Vec<String>> {
    if let Some(only) = &options.only {
        return Ok(vec![only.clone()]);
    }
    if let Some(packages) = &options.packages {
        let mut names = packages.clone();
        names.sort();
        names.dedup();
        return Ok(names);
    }
    Ok(Vec::new())
}

#[cfg(test)]
mod native_binary_tests {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use boost_fs::temp::create_folder;

    use super::{has_staged_native_binary, is_native_binary_package};
    use crate::workspace::{NpmPackage, PackageManifest};

    fn package_with_dir(name: &str, dir: PathBuf, os: Option<Vec<String>>, cpu: Option<Vec<String>>) -> NpmPackage {
        let manifest = PackageManifest {
            name: name.to_string(),
            version: "0.0.0".to_string(),
            private: false,
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            os,
            cpu,
            publish_config: None,
        };
        NpmPackage {
            name: name.to_string(),
            version: "0.0.0".to_string(),
            dir: dir.clone(),
            manifest_path: dir.join("package.json"),
            private: false,
            manifest,
        }
    }

    fn package(name: &str, os: Option<Vec<String>>, cpu: Option<Vec<String>>) -> NpmPackage {
        package_with_dir(name, PathBuf::from(name), os, cpu)
    }

    #[test]
    fn skills_is_not_native_binary_package() {
        assert!(!is_native_binary_package(&package("@doki-land/nifty-skills", None, None)));
    }

    #[test]
    fn platform_shard_requires_os_and_cpu() {
        assert!(is_native_binary_package(&package(
            "@doki-land/nifty-win32-x64",
            Some(vec!["win32".to_string()]),
            Some(vec!["x64".to_string()]),
        )));
    }

    #[test]
    fn staged_native_binary_supports_lib_and_root_layouts() {
        let root = create_folder().expect("tempdir");
        let lib_pkg = package_with_dir("lib", root.path().join("lib-pkg"), None, None);
        fs::create_dir_all(&lib_pkg.dir).expect("mkdir");
        assert!(!has_staged_native_binary(&lib_pkg));
        fs::create_dir_all(lib_pkg.dir.join("lib")).expect("mkdir lib");
        fs::write(lib_pkg.dir.join("lib").join("addon.node"), b"bin").expect("write lib addon");
        assert!(has_staged_native_binary(&lib_pkg));

        let root_pkg = package_with_dir("root", root.path().join("root-pkg"), None, None);
        fs::create_dir_all(&root_pkg.dir).expect("mkdir");
        assert!(!has_staged_native_binary(&root_pkg));
        fs::write(root_pkg.dir.join("addon.node"), b"bin").expect("write root addon");
        assert!(has_staged_native_binary(&root_pkg));
    }
}

#[cfg(test)]
mod error_classification_tests {
    use super::{PublishErrorKind, classify_publish_error};

    #[test]
    fn staged_is_not_live() {
        assert_eq!(
            classify_publish_error("npm ERR! Cannot publish over previously staged version of this package."),
            PublishErrorKind::StagedPending
        );
        assert_eq!(
            classify_publish_error("E409 publish failed: previously staged version exists"),
            PublishErrorKind::StagedPending
        );
    }

    #[test]
    fn previously_published_is_live() {
        assert_eq!(
            classify_publish_error("npm ERR! Cannot publish over the previously published version"),
            PublishErrorKind::AlreadyLive
        );
        assert_eq!(classify_publish_error("version already exists"), PublishErrorKind::AlreadyLive);
        assert_eq!(classify_publish_error("package has already been published"), PublishErrorKind::AlreadyLive);
    }

    #[test]
    fn network_or_auth_failure_is_other() {
        assert_eq!(classify_publish_error("npm ERR! code ENEEDAUTH"), PublishErrorKind::Other);
        assert_eq!(classify_publish_error("ETIMEDOUT registry.npmjs.org"), PublishErrorKind::Other);
        assert_eq!(classify_publish_error("package.json not found"), PublishErrorKind::Other);
    }
}

#[cfg(test)]
mod publish_filter_tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::resolve_candidate_packages;
    use crate::workspace::{NpmPackage, PackageManifest, PublishConfig};

    fn package(name: &str, private: bool) -> NpmPackage {
        let manifest = PackageManifest {
            name: name.to_string(),
            version: "0.0.0".to_string(),
            private,
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            os: None,
            cpu: None,
            publish_config: None,
        };
        NpmPackage {
            name: name.to_string(),
            version: "0.0.0".to_string(),
            dir: PathBuf::from(name),
            manifest_path: PathBuf::from(name).join("package.json"),
            private,
            manifest,
        }
    }

    fn package_with_deps(name: &str, private: bool, deps: &[(&str, &str)]) -> NpmPackage {
        let mut pkg = package(name, private);
        for (dep_name, spec) in deps {
            pkg.manifest.dependencies.insert(dep_name.to_string(), spec.to_string());
        }
        pkg
    }

    fn package_with_publish_name(workspace_name: &str, registry_name: &str, private: bool) -> NpmPackage {
        let mut package = package(workspace_name, private);
        package.manifest.publish_config = Some(PublishConfig { name: Some(registry_name.to_string()), access: None });
        package
    }

    #[test]
    fn default_skips_private_packages() {
        let packages = vec![package("@scope/main", false), package("@scope/native", true)];
        let by_name = packages.iter().map(|package| (package.name.clone(), package.clone())).collect::<BTreeMap<_, _>>();
        let (candidates, skipped) = resolve_candidate_packages(&packages, &by_name, &[]).expect("resolve");
        assert_eq!(candidates.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), vec!["@scope/main"]);
        assert_eq!(skipped, vec!["@scope/native".to_string()]);
    }

    #[test]
    fn explicit_targets_skip_private_packages() {
        let packages = vec![package("@scope/main", false), package("@scope/native", true)];
        let by_name = packages.iter().map(|package| (package.name.clone(), package.clone())).collect::<BTreeMap<_, _>>();
        let (candidates, skipped) =
            resolve_candidate_packages(&packages, &by_name, &["@scope/native".to_string()]).expect("resolve");
        assert!(candidates.is_empty());
        assert_eq!(skipped, vec!["@scope/native".to_string()]);
    }

    #[test]
    fn required_private_dependency_blocks_dependent() {
        let private = package("@scope/private", true);
        let main = package_with_deps("@scope/main", false, &[("@scope/private", "workspace:*")]);
        let packages = vec![main, private];
        let by_name = packages.iter().map(|package| (package.name.clone(), package.clone())).collect::<BTreeMap<_, _>>();
        let (candidates, skipped) = resolve_candidate_packages(&packages, &by_name, &[]).expect("resolve");
        assert!(candidates.is_empty());
        assert_eq!(skipped.len(), 2);
    }

    #[test]
    fn explicit_targets_resolve_publish_config_name() {
        let packages = vec![package_with_publish_name("vmz", "@vmz/vmz", false)];
        let by_name = packages.iter().map(|package| (package.name.clone(), package.clone())).collect::<BTreeMap<_, _>>();
        let (candidates, _) = resolve_candidate_packages(&packages, &by_name, &["@vmz/vmz".to_string()]).expect("resolve");
        assert_eq!(candidates.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), vec!["vmz"]);
    }
}

fn publish_package(
    package: &NpmPackage,
    by_name: &BTreeMap<String, NpmPackage>,
    dry_run: bool,
    tag: Option<&str>,
    access: Option<&str>,
    npm: Option<&Path>,
    auth: &OtpAuth,
    publish_version: Option<&str>,
    placeholder_dep_names: Option<&BTreeSet<String>>,
) -> Result<()> {
    let original = fs::read_to_string(&package.manifest_path).map_err(|err| err.to_string())?;
    let next = manifest::patch_manifest_for_publish(
        &original,
        package,
        by_name,
        manifest::PatchPublishOptions { publish_version, placeholder_dep_names },
    )?;

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
