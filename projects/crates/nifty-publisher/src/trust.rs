use serde_json::Value;

use crate::cache::PlaceholderCache;
use crate::npm::NpmRunner;
use crate::otp::{OtpAuth, OtpOverrides};
use crate::workspace::{find_workspace_root, list_workspace_packages};
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
    pub npm: Option<std::path::PathBuf>,
    pub otp: OtpOverrides,
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
    let auth = OtpAuth::load(&root, options.otp);
    let mut cache = PlaceholderCache::load(&root);
    let has_otp = auth.has_otp();
    let runner = NpmRunner::new(options.npm.as_deref(), auth);
    let packages = list_workspace_packages(&root)?;
    let names = packages
        .iter()
        .filter(|package| !package.private)
        .map(|package| package.name.clone())
        .filter(|name| options.only.as_ref().map(|only| only == name).unwrap_or(true))
        .collect::<Vec<_>>();

    if let Some(only) = &options.only {
        if !names.iter().any(|name| name == only) {
            return Err(format!("--only {only} is not a workspace package"));
        }
    }

    let mut report = TrustReport {
        root: root.clone(),
        configured: Vec::new(),
        skipped: Vec::new(),
        failed: Vec::new(),
    };

    for name in names {
        match configure_trust(&runner, &mut cache, &name, options.dry_run, options.refresh, has_otp) {
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

enum TrustOutcome {
    Configured,
    Skipped,
}

fn configure_trust(
    runner: &NpmRunner,
    cache: &mut PlaceholderCache,
    package: &str,
    dry_run: bool,
    refresh: bool,
    has_otp: bool,
) -> Result<TrustOutcome> {
    if cache.trust_matches_cached(package, refresh) {
        println!("trusted publisher already configured for {package} (cache)");
        return Ok(TrustOutcome::Skipped);
    }

    if !has_otp {
        return Err(
            "npm trust requires 2FA for live trust list: set NPM_TOTP_SECRET in .env.placeholder.local or pass --otp / --totp-secret"
                .into(),
        );
    }

    let list = runner.run(&["trust", "list", package, "--json"], None)?;
    if list.status != 0 {
        let blob = format!("{}\n{}", list.stdout, list.stderr);
        if blob.contains("EOTP") || blob.contains("one-time password") {
            return Err("npm requested OTP (EOTP)".into());
        }
        return Err(blob);
    }

    let configs = parse_trust_list(&list.stdout)?;
    cache.record_trust_list(package, configs.clone());
    let classification = crate::cache::classify_configs(&configs);
    if classification.matches {
        return Ok(TrustOutcome::Skipped);
    }

    if dry_run {
        println!("would configure trusted publisher for {package}");
        return Ok(TrustOutcome::Configured);
    }

    let args = [
        "trust",
        "github",
        package,
        &format!("--file={TRUST_FILE}"),
        &format!("--repo={TRUST_REPO}"),
        &format!("--env={TRUST_ENV}"),
        "--allow-publish",
        "--allow-stage-publish",
        "--yes",
    ];
    let created = runner.run(&args, None)?;
    if created.status != 0 {
        return Err(format!("{}\n{}", created.stdout, created.stderr));
    }
    println!("configured trusted publisher for {package}");
    Ok(TrustOutcome::Configured)
}

fn parse_trust_list(stdout: &str) -> Result<Vec<Value>> {
    let data: Value = serde_json::from_str(stdout).map_err(|err| err.to_string())?;
    if let Some(array) = data.as_array() {
        return Ok(array.clone());
    }
    if let Some(array) = data.get("configurations").and_then(Value::as_array) {
        return Ok(array.clone());
    }
    if let Some(array) = data.get("items").and_then(Value::as_array) {
        return Ok(array.clone());
    }
    if data.is_object() {
        return Ok(vec![data]);
    }
    Ok(Vec::new())
}

