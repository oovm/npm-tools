use crate::cache::PlaceholderCache;
use crate::otp::{OtpAuth, OtpOverrides};
use crate::registry_trust::{RegistryTrustClient, TrustCreateOutcome};
use crate::trust_expect::{resolve_trust_expect, TrustExpectInput};
use crate::workspace::{
    find_package_by_target, find_workspace_root, list_workspace_packages, registry_name,
    unpublishable_package_names,
};
use crate::Result;

pub const TRUST_REPO: &str = "oovm/npm-tools";
pub const TRUST_FILE: &str = "publish-npm.yml";
pub const TRUST_ENV: &str = "NPM_PUBLISH";

#[derive(Debug, Clone, Default)]
pub struct TrustOptions {
    pub cwd: Option<std::path::PathBuf>,
    pub dry_run: bool,
    pub refresh: bool,
    pub only: Option<String>,
    pub packages: Option<Vec<String>>,
    pub npm: Option<std::path::PathBuf>,
    pub otp: OtpOverrides,
    pub trust: Option<TrustExpectInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustReport {
    pub root: std::path::PathBuf,
    pub configured: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
}

pub fn trust_workspace(options: TrustOptions) -> Result<TrustReport> {
    let cwd = options
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let root = find_workspace_root(&cwd)?;
    let expect = resolve_trust_expect(&root, options.trust.as_ref());
    let names = resolve_trust_package_names(&root, &options)?;
    let auth = OtpAuth::load(&root, options.otp);
    let client = RegistryTrustClient::from_workspace(&auth, &root)?;
    let mut cache = PlaceholderCache::load(&root, &expect);

    let mut report = TrustReport {
        root: root.clone(),
        configured: Vec::new(),
        skipped: Vec::new(),
        failed: Vec::new(),
    };

    for name in names {
        match configure_trust(&client, &mut cache, &expect, &name, options.dry_run, options.refresh) {
            Ok(TrustOutcome::Configured) => report.configured.push(name),
            Ok(TrustOutcome::Skipped) => report.skipped.push(name),
            Err(message) => {
                eprintln!("trust failed for {name}: {message}");
                report.failed.push(name);
            }
        }
    }

    cache.save(&root)?;

    if !report.failed.is_empty() {
        return Err(format!("trust failed for {} package(s)", report.failed.len()));
    }
    Ok(report)
}

fn resolve_trust_package_names(root: &std::path::Path, options: &TrustOptions) -> Result<Vec<String>> {
    let workspace = list_workspace_packages(root)?;
    let by_name = workspace
        .iter()
        .map(|package| (package.name.clone(), package.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let blocked = unpublishable_package_names(&workspace, &by_name);

    let mut names: Vec<String> = if let Some(list) = &options.packages {
        list.iter()
            .filter(|target| {
                if let Some(package) = find_package_by_target(&by_name, target) {
                    if blocked.contains(&package.name) {
                        println!(
                            "skip trust target {target} (private workspace package or depends on one)"
                        );
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    } else {
        workspace
            .iter()
            .filter(|package| !blocked.contains(&package.name))
            .map(|package| registry_name(package).to_string())
            .collect()
    };
    names.sort();
    names.dedup();

    if let Some(only) = &options.only {
        if !names.iter().any(|name| name == only) {
            return Err(format!("--only {only} is not in publish.packages"));
        }
        return Ok(vec![only.clone()]);
    }
    Ok(names)
}

enum TrustOutcome {
    Configured,
    Skipped,
}

fn configure_trust(
    client: &RegistryTrustClient,
    cache: &mut PlaceholderCache,
    expect: &crate::cache::TrustExpect,
    package: &str,
    dry_run: bool,
    refresh: bool,
) -> Result<TrustOutcome> {
    if cache.trust_matches_cached(package, refresh) {
        println!("trusted publisher already configured for {package} (cache)");
        return Ok(TrustOutcome::Skipped);
    }

    let configs = match client.list(package) {
        Ok(configs) => configs,
        Err(err) if refresh => {
            eprintln!("warn: registry trust list failed for {package}: {err}; continuing with --refresh");
            Vec::new()
        }
        Err(err) => return Err(err),
    };
    cache.record_trust_list(package, configs.clone());
    if crate::registry_trust::trust_already_matches(&configs, expect) {
        return Ok(TrustOutcome::Skipped);
    }

    if dry_run {
        println!("would configure trusted publisher for {package}");
        return Ok(TrustOutcome::Configured);
    }

    match client.create(package, expect)? {
        TrustCreateOutcome::Created => {
            println!("configured trusted publisher for {package}");
            Ok(TrustOutcome::Configured)
        }
        TrustCreateOutcome::AlreadyExists => {
            println!("trusted publisher already exists for {package}");
            Ok(TrustOutcome::Skipped)
        }
    }
}
