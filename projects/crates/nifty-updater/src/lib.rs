//! Update cargo and npm dependencies in hybrid Nifty projects.

mod cargo;
mod npm;
mod registry;
mod version;

use std::path::{Path, PathBuf};

use nifty_config::{ProjectKind, ProjectLayout, detect_project_layout};

pub use cargo::CargoUpgrade;
pub use npm::{NpmOutdated, discover_package_dirs};

pub type Result<T> = std::result::Result<T, String>;

/// Options for [`run_update`].
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    /// Prompt before applying each ecosystem's updates.
    pub interactive: bool,
    /// Directory to detect the project layout from.
    pub cwd: Option<PathBuf>,
}

/// Update dependencies for the detected cargo/npm/hybrid project.
pub fn run_update(options: UpdateOptions) -> Result<()> {
    let cwd = options.cwd.clone().unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let layout = detect_project_layout(&cwd);

    match layout.kind {
        ProjectKind::Unknown => {
            return Err(format!("no Cargo.toml or package.json found from {}", cwd.display()));
        }
        ProjectKind::Cargo => update_cargo(&layout, options.interactive)?,
        ProjectKind::Npm => update_npm(&layout, options.interactive)?,
        ProjectKind::Hybrid => {
            update_cargo(&layout, options.interactive)?;
            update_npm(&layout, options.interactive)?;
        }
    }

    Ok(())
}

fn update_cargo(layout: &ProjectLayout, interactive: bool) -> Result<()> {
    let root = layout
        .cargo_workspace_root
        .clone()
        .or_else(|| layout.cargo_manifest.as_ref().and_then(|path| path.parent().map(Path::to_path_buf)))
        .ok_or_else(|| "cargo project not found".to_string())?;

    let upgrades = cargo::plan_cargo_upgrades(&root)?;
    if upgrades.is_empty() {
        println!("cargo: dependencies already up to date");
        return Ok(());
    }

    if interactive {
        let selected = cargo::select_upgrades(&upgrades)?;
        if selected.is_empty() {
            println!("cargo: no upgrades selected");
            return Ok(());
        }
        cargo::apply_cargo_upgrades(&root, &selected)?;
    }
    else {
        cargo::apply_cargo_upgrades_all(&root)?;
    }

    Ok(())
}

fn update_npm(layout: &ProjectLayout, interactive: bool) -> Result<()> {
    let packages = npm::discover_package_dirs(layout)?;
    if packages.is_empty() {
        println!("js: no package.json directories found");
        return Ok(());
    }

    let mut pending_install_root: Option<PathBuf> = None;
    for package_dir in packages {
        if let Some(root) = update_npm_dir(&package_dir, interactive)? {
            pending_install_root = Some(root);
        }
    }

    if let Some(root) = pending_install_root {
        npm::sync_lockfile_at(&root)?;
    }

    Ok(())
}

fn update_npm_dir(package_dir: &Path, interactive: bool) -> Result<Option<PathBuf>> {
    let outdated = npm::list_outdated(package_dir)?;
    if outdated.is_empty() {
        println!("js ({}): dependencies already up to date", package_dir.display());
        return Ok(None);
    }

    if interactive {
        let selected = npm::select_outdated(&outdated, package_dir)?;
        if selected.is_empty() {
            println!("js ({}): no upgrades selected", package_dir.display());
            return Ok(None);
        }
        npm::apply_npm_upgrades(package_dir, &selected, false)?;
    }
    else {
        npm::apply_npm_upgrades(package_dir, &outdated, false)?;
    }

    Ok(Some(npm::package_manager_root(package_dir)))
}
