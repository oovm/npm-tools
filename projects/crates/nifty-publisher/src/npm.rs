use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
        let output = run_command(&self.program, &argv, cwd, true, &self.auth)?;
        if output.status != 0 {
            return Err(format!("npm command failed (exit {})", output.status));
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
    let user_config = write_user_npmrc(auth)?;
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    apply_auth_env(&mut command, auth, user_config.as_deref());
    if inherit {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        let status = command.status().map_err(|err| err.to_string())?;
        cleanup_user_npmrc(user_config);
        return Ok(NpmOutput {
            status: status.code().unwrap_or(1),
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let output = command.output().map_err(|err| err.to_string())?;
    cleanup_user_npmrc(user_config);
    Ok(NpmOutput {
        status: output.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(windows)]
fn try_run_via_cmd(program: &Path, args: &[String], cwd: Option<&Path>, inherit: bool, auth: &OtpAuth) -> Result<NpmOutput> {
    let user_config = write_user_npmrc(auth)?;
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
        cleanup_user_npmrc(user_config);
        return Ok(NpmOutput {
            status: status.code().unwrap_or(1),
            stdout: String::new(),
            stderr: String::new(),
        });
    }
    let output = command.output().map_err(|err| err.to_string())?;
    cleanup_user_npmrc(user_config);
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

fn write_user_npmrc(auth: &OtpAuth) -> Result<Option<PathBuf>> {
    if let Some(token) = &auth.token {
        let path = std::env::temp_dir().join(format!("nifty-npmrc-{}", std::process::id()));
        std::fs::write(&path, format!("//registry.npmjs.org/:_authToken={token}\n")).map_err(|err| err.to_string())?;
        return Ok(Some(path));
    }
    Ok(None)
}

fn cleanup_user_npmrc(path: Option<PathBuf>) {
    if let Some(path) = path {
        let _ = std::fs::remove_file(path);
    }
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
