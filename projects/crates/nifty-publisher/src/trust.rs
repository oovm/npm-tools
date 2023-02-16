use serde_json::Value;

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
    if !auth.has_otp() {
        return Err(
            "npm trust requires 2FA: set NPM_TOTP_SECRET in .env.placeholder.local or pass --otp / --totp-secret"
                .into(),
        );
    }

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
        root,
        configured: Vec::new(),
        skipped: Vec::new(),
        failed: Vec::new(),
    };

    for name in names {
        match configure_trust(&runner, &name, options.dry_run) {
            Ok(TrustOutcome::Configured) => report.configured.push(name),
            Ok(TrustOutcome::Skipped) => report.skipped.push(name),
            Err(message) => {
                eprintln!("trust failed for {name}: {message}");
                report.failed.push(name);
            }
        }
    }

    if !report.failed.is_empty() {
        return Err(format!("trust failed for {} package(s)", report.failed.len()));
    }
    Ok(report)
}

enum TrustOutcome {
    Configured,
    Skipped,
}

fn configure_trust(runner: &NpmRunner, package: &str, dry_run: bool) -> Result<TrustOutcome> {
    let list = runner.run(&["trust", "list", package, "--json"], None)?;
    if list.status != 0 {
        let blob = format!("{}\n{}", list.stdout, list.stderr);
        if blob.contains("EOTP") || blob.contains("one-time password") {
            return Err("npm requested OTP (EOTP)".into());
        }
        return Err(blob);
    }

    let configs = parse_trust_list(&list.stdout)?;
    let classification = classify_configs(&configs);
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

struct TrustClassification {
    matches: bool,
}

fn classify_configs(configs: &[Value]) -> TrustClassification {
    if configs.iter().any(trust_exact) {
        return TrustClassification { matches: true };
    }
    if configs.iter().any(trust_matches) {
        return TrustClassification { matches: true };
    }
    TrustClassification { matches: false }
}

fn trust_exact(config: &Value) -> bool {
    trust_matches(config) && trust_fields(config).env == TRUST_ENV
}

fn trust_matches(config: &Value) -> bool {
    if let Some(raw) = config.get("raw").and_then(|value| value.as_str()) {
        return raw.contains(TRUST_REPO) && raw.contains(TRUST_FILE) && raw.contains(TRUST_ENV);
    }
    let fields = trust_fields(config);
    fields.repo == TRUST_REPO && fields.file == TRUST_FILE
}

struct TrustFields {
    repo: String,
    file: String,
    env: String,
}

fn trust_fields(config: &Value) -> TrustFields {
    let claims = config.get("claims").unwrap_or(config);
    TrustFields {
        repo: pick_string(config, claims, &["repository", "repo"]),
        file: pick_string(config, claims, &["file", "workflow", "workflowFile"]),
        env: pick_string(config, claims, &["environment", "env"]),
    }
}

fn pick_string(config: &Value, claims: &Value, keys: &[&str]) -> String {
    for &key in keys {
        if let Some(value) = config.get(key).and_then(|value| value.as_str()) {
            return value.to_string();
        }
        if let Some(value) = claims.get(key).and_then(|value| value.as_str()) {
            return value.to_string();
        }
        if let Some(workflow) = claims.get("workflow_ref").and_then(Value::as_object) {
            if let Some(value) = workflow.get(key).and_then(|value| value.as_str()) {
                return value.to_string();
            }
        }
    }
    String::new()
}
