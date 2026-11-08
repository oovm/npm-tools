//! Publish npm workspace packages in dependency order.

mod cache;
mod graph;
mod manifest;
mod npm;
mod npmrc;
mod otp;
mod publish;
mod registry_trust;
mod trust;
pub mod trust_expect;
mod workspace;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

pub use cache::{CACHE_DIR_NAME, CACHE_FILE_NAME, PlaceholderCache, TrustExpect};
pub use graph::plan_publish_order;
pub use otp::{OtpAuth, OtpOverrides};
pub use publish::{PackageFailure, PublishAttempt, PublishBatch, PublishFailure, PublishFailureKind};
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
    /// Packages whose version only reached the registry staging area (not live).
    pub staged: Vec<String>,
}

/// Discover workspace npm packages, sort them with `petgraph`, and run `npm publish`.
pub fn publish_workspace(options: PublishOptions) -> Result<PublishReport> {
    let cwd = options.cwd.clone().unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let root = find_workspace_root(&cwd)?;
    let publish_targets = resolve_publish_targets(&options)?;
    let auth = OtpAuth::load(&root, options.otp);
    let trust_expect = trust_expect::resolve_trust_expect(&root, options.trust.as_ref())?;
    let mut cache = cache::PlaceholderCache::load(&root, &trust_expect);
    let runner = npm::NpmRunner::new(options.npm.as_deref(), auth.clone());
    let packages = list_workspace_packages(&root)?;
    let by_name = packages.iter().map(|package| (package.name.clone(), package.clone())).collect::<BTreeMap<_, _>>();
    let (candidates, skipped) = resolve_candidate_packages(&packages, &by_name, &publish_targets)?;
    let publish_plan = candidates.iter().map(|package| package.name.clone()).collect::<BTreeSet<_>>();
    for package in &candidates {
        manifest::verify_publish_dependency_pins(package, &by_name, &publish_plan, options.version.as_deref())?;
    }
    let order = graph::sort_packages_for_publish(&candidates, &by_name)?;
    ensure_requested_targets_in_plan(&publish_targets, &order, &by_name)?;

    let access = options.access.as_deref().or(Some("public"));
    let mut skipped_versions = Vec::new();
    let mut batch = PublishBatch::default();

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
        if !options.placeholder && is_native_binary_package(package) {
            verify_native_release(package, &registry, &runner)?;
        }
        println!("publishing {registry}@{publish_version}");
        let attempt = match publish_package(
            package,
            &by_name,
            options.dry_run,
            options.tag.as_deref(),
            access,
            options.npm.as_deref(),
            &auth,
            if options.placeholder { Some(PLACEHOLDER_VERSION) } else { Some(publish_version) },
            if options.placeholder { Some(&placeholder_names) } else { None },
        ) {
            Ok(attempt) => attempt,
            Err(message) => PublishAttempt::Failed(PublishFailure::local(message)),
        };
        match &attempt {
            PublishAttempt::Published => {}
            PublishAttempt::AlreadyLive => {
                println!("skip publish {name}@{publish_version} (registry says already live)");
            }
            PublishAttempt::StagedPending => {
                println!("skip publish {name}@{publish_version} (registry has a staged pending version — not yet live)");
            }
            PublishAttempt::Failed(failure) => {
                println!("publish failed for {name}: {}", failure.summary());
            }
        }
        if batch.record(&mut cache, name, publish_version, attempt, options.dry_run) {
            cache.save(&root)?;
        }
    }

    cache.save(&root)?;

    if let Some(summary) = batch.failure_summary() {
        return Err(summary);
    }

    skipped_versions.extend(batch.already_live);
    Ok(PublishReport { root, order, published: batch.published, skipped, skipped_versions, staged: batch.staged })
}

fn is_native_binary_package(package: &NpmPackage) -> bool {
    package.manifest.os.as_ref().is_some_and(|os| !os.is_empty())
        && package.manifest.cpu.as_ref().is_some_and(|cpu| !cpu.is_empty())
}

/// A native platform package must stage exactly the addon its manifest ships.
///
/// The meta package lists these platform packages as optional dependencies, so a
/// release that silently drops one installs without native bindings on that
/// platform. Publishing stops before the dependent packages are reached, because
/// the plan always orders platform packages first.
fn verify_native_release(package: &NpmPackage, registry: &str, runner: &npm::NpmRunner) -> Result<()> {
    let entry = native_entry(package, registry)?;
    let relative =
        entry.strip_prefix(&package.dir).map_err(|_| format!("native entry for {registry} escapes its package directory"))?;
    if !entry.is_file() {
        return Err(format!(
            "missing staged native binary `{}` for {registry} — stage every platform with `nifty install-native --from <dir>` before publishing",
            relative.display()
        ));
    }
    let plan = runner.pack_plan(&package.dir)?;
    let relative = relative.to_string_lossy().replace('\\', "/");
    if !plan.iter().any(|path| path == &relative) {
        return Err(format!(
            "native binary `{relative}` for {registry} is not included in the tarball — check `files` and `.npmignore` before publishing"
        ));
    }
    Ok(())
}

/// Expected native entry, derived from the package manifest `main`.
///
/// Native packages ship the addon itself as their module entry, so `main` is the
/// single source for the staged path, the tarball check and the runtime loader.
fn native_entry(package: &NpmPackage, registry: &str) -> Result<PathBuf> {
    let main = package
        .manifest
        .main
        .as_deref()
        .ok_or_else(|| format!("native package {registry} must declare its addon as `main`"))?;
    if !main.ends_with(".node") {
        return Err(format!("native package {registry} must declare a `.node` entry as `main`, got `{main}`"));
    }
    let relative = Path::new(main);
    if relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir))
    {
        return Err(format!("native package {registry} has an invalid `main` path `{main}`"));
    }
    Ok(package.dir.join(relative))
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
            return Err(format!(
                "publish target `{}` is private or depends on a private package — remove `private` to publish it",
                package.name
            ));
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

/// Every explicitly requested target must appear in the resolved publish plan.
///
/// A requested package that silently disappears from the plan used to leave the CLI exit code at 0,
/// which made a release workflow look successful while publishing nothing. Missing targets are a
/// hard error with the package name and the reason.
fn ensure_requested_targets_in_plan(
    targets: &[String],
    order: &[String],
    by_name: &BTreeMap<String, NpmPackage>,
) -> Result<()> {
    use crate::workspace::find_package_by_target;

    if targets.is_empty() {
        return Ok(());
    }
    let planned = order.iter().collect::<BTreeSet<_>>();
    let mut missing = Vec::new();
    for target in targets {
        match find_package_by_target(by_name, target) {
            None => missing.push(format!("`{target}`: not found in this workspace")),
            Some(package) if !planned.contains(&package.name) => {
                missing.push(format!("`{}`: resolved out of the publish plan", package.name));
            }
            Some(_) => {}
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    Err(format!("requested publish targets are missing from the plan:\n  {}", missing.join("\n  ")))
}

#[cfg(test)]
mod plan_guard_tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::workspace::{NpmPackage, PackageManifest, collect_internal_dependency_names};

    use super::{ensure_requested_targets_in_plan, resolve_candidate_packages};

    fn package(name: &str, dependencies: &[(&str, &str)]) -> NpmPackage {
        let dependencies =
            dependencies.iter().map(|(key, value)| ((*key).to_string(), (*value).to_string())).collect::<BTreeMap<_, _>>();
        let manifest = PackageManifest {
            name: name.to_string(),
            version: "0.0.1".to_string(),
            private: false,
            dependencies: dependencies.clone(),
            dev_dependencies: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            os: None,
            cpu: None,
            main: None,
            publish_config: None,
        };
        NpmPackage {
            name: name.to_string(),
            version: "0.0.1".to_string(),
            dir: PathBuf::from(name),
            manifest_path: PathBuf::from(name).join("package.json"),
            private: false,
            manifest,
        }
    }

    #[test]
    fn a_workspace_package_that_depends_on_a_missing_package_still_fails_by_name() {
        // `@scope/app` declares `@scope/ghost`, which is not a workspace package.
        // Requesting `@scope/app` must not silently vanish from the plan.
        let packages = vec![package("@scope/app", &[("@scope/ghost", "0.0.1")])];
        let by_name = packages.iter().map(|entry| (entry.name.clone(), entry.clone())).collect::<BTreeMap<_, _>>();

        assert!(collect_internal_dependency_names(&packages[0], &by_name).is_empty());

        let error = ensure_requested_targets_in_plan(&["@scope/app".to_string()], &[], &by_name).expect_err("must fail");
        assert!(error.contains("@scope/app"), "error must name the requested package: {error}");
        assert!(error.contains("missing from the plan"), "error must explain why: {error}");
    }

    #[test]
    fn unknown_target_is_reported_by_name() {
        let packages = vec![package("@scope/app", &[])];
        let by_name = packages.iter().map(|entry| (entry.name.clone(), entry.clone())).collect::<BTreeMap<_, _>>();
        let error = ensure_requested_targets_in_plan(&["@scope/ghost".to_string()], &[], &by_name).expect_err("must fail");
        assert!(error.contains("@scope/ghost"), "error must name the unknown target: {error}");
    }

    #[test]
    fn every_requested_target_must_be_planned() {
        let packages = vec![package("@scope/app", &[]), package("@scope/platform", &[])];
        let by_name = packages.iter().map(|entry| (entry.name.clone(), entry.clone())).collect::<BTreeMap<_, _>>();
        let order = vec!["@scope/platform".to_string(), "@scope/app".to_string()];
        ensure_requested_targets_in_plan(&["@scope/app".to_string(), "@scope/platform".to_string()], &order, &by_name)
            .expect("all targets are planned");
    }

    #[test]
    fn private_target_is_an_error_instead_of_a_silent_skip() {
        let mut packages = vec![package("@scope/app", &[])];
        packages[0].private = true;
        packages[0].manifest.private = true;
        let by_name = packages.iter().map(|entry| (entry.name.clone(), entry.clone())).collect::<BTreeMap<_, _>>();
        let error =
            resolve_candidate_packages(&packages, &by_name, &["@scope/app".to_string()]).expect_err("private target must fail");
        assert!(error.contains("@scope/app"), "error must name the package: {error}");
        assert!(error.contains("private"), "error must explain why: {error}");
    }
}

#[cfg(test)]
mod native_binary_tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use super::{is_native_binary_package, native_entry};
    use crate::workspace::{NpmPackage, PackageManifest};

    fn package(name: &str, os: Option<Vec<String>>, cpu: Option<Vec<String>>, main: Option<&str>) -> NpmPackage {
        let dir = PathBuf::from(name);
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
            main: main.map(str::to_string),
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

    fn shard(main: Option<&str>) -> NpmPackage {
        package("@doki-land/nifty-win32-x64", Some(vec!["win32".to_string()]), Some(vec!["x64".to_string()]), main)
    }

    #[test]
    fn skills_is_not_native_binary_package() {
        assert!(!is_native_binary_package(&package("@doki-land/nifty-skills", None, None, None)));
    }

    #[test]
    fn platform_shard_requires_os_and_cpu() {
        assert!(is_native_binary_package(&shard(Some("./lib/index.win32-x64-msvc.node"))));
    }

    #[test]
    fn native_entry_comes_from_the_manifest_main() {
        let entry = native_entry(&shard(Some("./lib/index.win32-x64-msvc.node")), "@doki-land/nifty-win32-x64")
            .expect("declared addon");
        assert_eq!(entry, PathBuf::from("@doki-land/nifty-win32-x64").join("lib").join("index.win32-x64-msvc.node"));
    }

    #[test]
    fn native_entry_must_be_a_declared_node_addon() {
        let missing = native_entry(&shard(None), "@doki-land/nifty-win32-x64").expect_err("main is required");
        assert!(missing.contains("`main`"), "error must point at the manifest entry: {missing}");

        let script = native_entry(&shard(Some("./lib/index.js")), "@doki-land/nifty-win32-x64").expect_err("addon only");
        assert!(script.contains(".node"), "error must explain the addon requirement: {script}");

        let escaping = native_entry(&shard(Some("../addon.node")), "@doki-land/nifty-win32-x64").expect_err("relative only");
        assert!(escaping.contains("../addon.node"), "error must name the invalid path: {escaping}");
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
            main: None,
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
    fn explicit_private_target_fails_by_name() {
        let packages = vec![package("@scope/main", false), package("@scope/native", true)];
        let by_name = packages.iter().map(|package| (package.name.clone(), package.clone())).collect::<BTreeMap<_, _>>();
        let error = resolve_candidate_packages(&packages, &by_name, &["@scope/native".to_string()])
            .expect_err("an explicitly requested private package must fail");
        assert!(error.contains("@scope/native"), "error must name the package: {error}");
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
) -> Result<PublishAttempt> {
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

    let attempt = match npm::run_npm_publish(&package.dir, dry_run, tag, access, npm, auth) {
        Ok(()) => PublishAttempt::Published,
        Err(failure) => PublishAttempt::from_failure(failure),
    };
    restore();
    Ok(attempt)
}
