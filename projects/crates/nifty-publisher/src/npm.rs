use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::npmrc;
use crate::otp::OtpAuth;
use crate::Result;

pub struct NpmOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct NpmRunner {
    program: PathBuf,
    auth: OtpAuth,
}

impl NpmRunner {
    pub fn new(program: Option<&Path>, auth: OtpAuth) -> Self {
        Self {
            program: resolve_npm_executable(program),
            auth,
        }
    }

    pub fn run(&self, args: &[&str], cwd: Option<&Path>) -> Result<NpmOutput> {
        let argv = with_otp(args, &self.auth);
        run_command(&self.program, &argv, cwd, false, &self.auth)
    }

    pub fn run_inherit(&self, args: &[&str], cwd: Option<&Path>) -> Result<()> {
        let argv = with_otp(args, &self.auth);
        let output = run_command(&self.program, &argv, cwd, false, &self.auth)?;
        if output.status != 0 {
            return Err(format_npm_failure(&argv, &output));
        }
        if !output.stdout.is_empty() {
            print!("{}", output.stdout);
        }
        if !output.stderr.is_empty() {
            eprint!("{}", output.stderr);
        }
        Ok(())
    }

    pub fn view_version(&self, name: &str) -> Result<Option<String>> {
        let output = self.run(&["view", name, "version"], None)?;
        if output.status != 0 {
            return Ok(None);
        }
        let version = output.stdout.trim();
        if version.is_empty() {
            Ok(None)
        } else {
            Ok(Some(version.to_string()))
        }
    }
}

pub fn run_npm_publish(
    dir: &Path,
    dry_run: bool,
    tag: Option<&str>,
    access: Option<&str>,
    npm: Option<&Path>,
    auth: &OtpAuth,
) -> Result<()> {
    let runner = NpmRunner::new(npm, auth.clone());
    let mut args = vec!["publish"];
    if dry_run {
        args.push("--dry-run");
    }
    if let Some(tag) = tag {
        args.push("--tag");
        args.push(tag);
    }
    if let Some(access) = access {
        args.push("--access");
        args.push(access);
    }
    runner.run_inherit(&args, Some(dir))
}

fn with_otp(args: &[&str], auth: &OtpAuth) -> Vec<String> {
    let mut argv = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    if let Some(code) = auth.current_otp() {
        argv.push(format!("--otp={code}"));
    }
    argv
}

fn run_command(program: &Path, args: &[String], cwd: Option<&Path>, inherit: bool, auth: &OtpAuth) -> Result<NpmOutput> {
    match try_run(program, args, cwd, inherit, auth) {
        Ok(output) => Ok(output),
        Err(err) if cfg!(windows) => try_run_via_cmd(program, args, cwd, inherit, auth)
            .map_err(|shell_err| format!("failed to run npm ({}): {err}; cmd: {shell_err}", program.display())),
        Err(err) => Err(format!("failed to run npm ({}): {err}", program.display())),
    }
}

fn try_run(program: &Path, args: &[String], cwd: Option<&Path>, inherit: bool, auth: &OtpAuth) -> Result<NpmOutput> {
    let user_config = npm_user_config(auth, cwd)?;
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    apply_auth_env(&mut command, auth, user_config.as_deref());
    if inherit {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        let status = command.status().map_err(|err| err.to_string())?;
        cleanup_temp_npmrc(user_config);
        return Ok(NpmOutput {
            status: status.code().unwrap_or(1),
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let output = command.output().map_err(|err| err.to_string())?;
    cleanup_temp_npmrc(user_config);
    Ok(NpmOutput {
        status: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(windows)]
fn try_run_via_cmd(program: &Path, args: &[String], cwd: Option<&Path>, inherit: bool, auth: &OtpAuth) -> Result<NpmOutput> {
    let user_config = npm_user_config(auth, cwd)?;
    let mut command = Command::new("cmd");
    command.arg("/C").arg(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    apply_auth_env(&mut command, auth, user_config.as_deref());
    if inherit {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        let status = command.status().map_err(|err| err.to_string())?;
        cleanup_temp_npmrc(user_config);
        return Ok(NpmOutput {
            status: status.code().unwrap_or(1),
            stdout: String::new(),
            stderr: String::new(),
        });
    }
    let output = command.output().map_err(|err| err.to_string())?;
    cleanup_temp_npmrc(user_config);
    Ok(NpmOutput {
        status: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(not(windows))]
fn try_run_via_cmd(_program: &Path, _args: &[String], _cwd: Option<&Path>, _inherit: bool, _auth: &OtpAuth) -> Result<NpmOutput> {
    Err("cmd fallback only on Windows".into())
}

fn apply_auth_env(command: &mut Command, auth: &OtpAuth, user_config: Option<&Path>) {
    if let Some(path) = user_config {
        command.env("NPM_CONFIG_USERCONFIG", path);
    }
    if let Some(token) = &auth.token {
        command.env("NODE_AUTH_TOKEN", token);
    }
}

fn npm_user_config(auth: &OtpAuth, cwd: Option<&Path>) -> Result<Option<PathBuf>> {
    let root = cwd
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    if let Some(path) = npmrc::npm_config_path(&root) {
        return Ok(Some(path));
    }
    if let Some(token) = auth.token.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        return write_temp_npmrc(token);
    }
    Ok(None)
}

fn write_temp_npmrc(token: &str) -> Result<Option<PathBuf>> {
    let path = std::env::temp_dir().join(format!("nifty-npmrc-{}", std::process::id()));
    std::fs::write(&path, format!("//registry.npmjs.org/:_authToken={token}\n"))
        .map_err(|err| err.to_string())?;
    Ok(Some(path))
}

fn cleanup_temp_npmrc(path: Option<PathBuf>) {
    if let Some(path) = path {
        if path.starts_with(std::env::temp_dir()) {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn format_npm_failure(argv: &[String], output: &NpmOutput) -> String {
    let blob = format!("{}\n{}", output.stdout, output.stderr);
    let joined = argv.join(" ");
    let status = output.status;
    if is_oidc_auth_failure(&blob) {
        return format!(
            "npm OIDC/trusted publish failed for `{joined}` (exit {status}). \
Configure Trusted Publisher: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/npm-tools. \
npm output:\n{blob}"
        );
    }
    if blob.trim().is_empty() {
        return format!("npm command failed (exit {status}): `{joined}`");
    }
    format!("npm command failed (exit {status}): `{joined}`\n{blob}")
}

fn is_oidc_auth_failure(blob: &str) -> bool {
    let lower = blob.to_ascii_lowercase();
    lower.contains("eneedauth")
        || lower.contains("unable to authenticate")
        || lower.contains("not authorized")
        || lower.contains("trusted publisher")
        || lower.contains("identity token")
        || lower.contains("do not have permission")
        || lower.contains("access token expired")
}

fn resolve_npm_executable(explicit: Option<&Path>) -> PathBuf {
    if let Some(path) = explicit {
        return path.to_path_buf();
    }
    if let Ok(path) = std::env::var("NIFTY_NPM") {
        return PathBuf::from(path);
    }
    if let Ok(node) = std::env::var("NODE") {
        let sibling = npm_next_to_node(&node);
        if sibling.is_file() {
            return sibling;
        }
    }
    if cfg!(windows) {
        PathBuf::from("npm.cmd")
    } else {
        PathBuf::from("npm")
    }
}

fn npm_next_to_node(node: &str) -> PathBuf {
    let node_path = PathBuf::from(node);
    let dir = node_path.parent().unwrap_or_else(|| Path::new("."));
    if cfg!(windows) {
        dir.join("npm.cmd")
    } else {
        dir.join("npm")
    }
}
