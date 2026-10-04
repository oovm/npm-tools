use std::{fs, path::Path, process::Command};

use crate::{Result, github::split_repo};

pub fn deploy_github_pages(repo: &str, token: &str, dir: &Path) -> Result<()> {
    let (owner, name) = split_repo(repo)?;
    let dir = dir.canonicalize().map_err(|err| format!("resolve pages dir {}: {err}", dir.display()))?;

    let work = tempfile::tempdir().map_err(|err| err.to_string())?;
    let root = work.path();

    copy_dir_recursive(&dir, root)?;
    let nojekyll = root.join(".nojekyll");
    if !nojekyll.exists() {
        fs::write(&nojekyll, b"").map_err(|err| err.to_string())?;
    }

    run_git(root, &["init"])?;
    run_git(root, &["config", "user.name", "github-actions[bot]"])?;
    run_git(root, &["config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com"])?;
    run_git(root, &["add", "."])?;
    run_git(root, &["commit", "-m", "Deploy GitHub Pages"])?;

    let remote = format!("https://x-access-token:{token}@github.com/{owner}/{name}.git");
    run_git(root, &["remote", "add", "origin", &remote])?;
    run_git(root, &["push", "--force", "origin", "HEAD:gh-pages"])?;

    println!("pages: pushed {} to gh-pages", dir.display());
    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(from).into_iter().filter_map(|entry| entry.ok()) {
        let path = entry.path();
        let relative = path.strip_prefix(from).map_err(|err| err.to_string())?;
        let target = to.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|err| err.to_string())?;
        }
        else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|err| err.to_string())?;
            }
            fs::copy(path, &target).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

fn run_git(cwd: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git").args(args).current_dir(cwd).status().map_err(|err| format!("run git: {err}"))?;
    if !status.success() {
        return Err(format!("git {} failed", args.join(" ")));
    }
    Ok(())
}
