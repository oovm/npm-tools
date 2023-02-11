use std::path::Path;
use std::process::Command;

use crate::Result;

pub fn run_npm_publish(dir: &Path, dry_run: bool, tag: Option<&str>, access: Option<&str>) -> Result<()> {
    let mut command = Command::new("npm");
    command.arg("publish").current_dir(dir);
    if dry_run {
        command.arg("--dry-run");
    }
    if let Some(tag) = tag {
        command.arg("--tag").arg(tag);
    }
    if let Some(access) = access {
        command.arg("--access").arg(access);
    }

    let status = command.status().map_err(|err| format!("failed to run npm publish: {err}"))?;
    if !status.success() {
        return Err(format!("npm publish failed in {}", dir.display()));
    }
    Ok(())
}
