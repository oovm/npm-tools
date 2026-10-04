use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use dialoguer::MultiSelect;
use nifty_config::ProjectLayout;
use serde_json::{Map, Value};

use crate::{
    Result,
    registry::fetch_npm_latest,
    version::{npm_spec_version, npm_upgrade_needed, npm_version_spec},
};

const DEPENDENCY_FIELDS: [&str; 4] = ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"];

/// One outdated JavaScript dependency from npm registry lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmOutdated {
    pub name: String,
    pub current: String,
    pub wanted: String,
    pub latest: String,
    pub package_dir: PathBuf,
}

pub fn discover_package_dirs(layout: &ProjectLayout) -> Result<Vec<PathBuf>> {
    if layout.root.join("pnpm-workspace.yaml").is_file() {
        return discover_pnpm_workspace_packages(&layout.root);
    }

    let mut roots = BTreeSet::new();
    if let Some(root) = &layout.npm_workspace_root {
        roots.insert(root.clone());
    }
    if let Some(manifest) = &layout.package_manifest {
        if let Some(parent) = manifest.parent() {
            roots.insert(parent.to_path_buf());
        }
    }
    Ok(roots.into_iter().collect())
}

pub fn list_outdated(package_dir: &Path) -> Result<Vec<NpmOutdated>> {
    let manifest_path = package_dir.join("package.json");
    if !manifest_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
    let value: Value = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    let object = value.as_object().ok_or_else(|| format!("{}: root must be an object", manifest_path.display()))?;

    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for field in DEPENDENCY_FIELDS {
        let Some(entries) = object.get(field).and_then(Value::as_object)
        else {
            continue;
        };
        for (name, spec_value) in entries {
            let spec = spec_value.as_str().ok_or_else(|| format!("dependency {name} must be a string"))?;
            if !is_registry_spec(spec) {
                continue;
            }
            if !seen.insert(name.clone()) {
                continue;
            }
            let latest = fetch_npm_latest(name)?;
            if !npm_upgrade_needed(spec, &latest)? {
                continue;
            }
            let current = npm_spec_version(spec).to_string();
            out.push(NpmOutdated {
                name: name.clone(),
                current,
                wanted: latest.clone(),
                latest,
                package_dir: package_dir.to_path_buf(),
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn select_outdated(outdated: &[NpmOutdated], package_dir: &Path) -> Result<Vec<NpmOutdated>> {
    let labels: Vec<String> = outdated
        .iter()
        .map(|item| format!("{} {} -> {} (latest {})", item.name, item.current, item.wanted, item.latest))
        .collect();
    let defaults = vec![true; labels.len()];
    let prompt = format!("Select JavaScript upgrades ({})", package_dir.display());
    let picked =
        MultiSelect::new().with_prompt(prompt).items(&labels).defaults(&defaults).interact().map_err(|err| err.to_string())?;

    Ok(picked.into_iter().map(|index| outdated[index].clone()).collect())
}

pub fn apply_npm_upgrades(package_dir: &Path, selected: &[NpmOutdated], sync_lock: bool) -> Result<()> {
    if selected.is_empty() {
        return Ok(());
    }
    let manifest_path = package_dir.join("package.json");
    let raw = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
    let mut value: Value = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    let object = value.as_object_mut().ok_or_else(|| format!("{}: root must be an object", manifest_path.display()))?;

    for item in selected {
        patch_package_json_dep(object, &item.name, &npm_version_spec(&item.latest))?;
        println!("js ({}): upgraded {} -> {}", package_dir.display(), item.name, item.latest);
    }

    fs::write(&manifest_path, format!("{}\n", serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?))
        .map_err(|err| err.to_string())?;
    if sync_lock {
        sync_lockfile(package_dir)?;
    }
    Ok(())
}

pub fn apply_npm_update_all(package_dir: &Path) -> Result<()> {
    let outdated = list_outdated(package_dir)?;
    apply_npm_upgrades(package_dir, &outdated, true)
}

pub fn package_manager_root(package_dir: &Path) -> PathBuf {
    find_package_manager_root(package_dir)
}

pub fn sync_lockfile_at(root: &Path) -> Result<()> {
    sync_lockfile(root)
}

fn patch_package_json_dep(object: &mut Map<String, Value>, name: &str, new_spec: &str) -> Result<()> {
    for field in DEPENDENCY_FIELDS {
        let Some(entries) = object.get_mut(field).and_then(Value::as_object_mut)
        else {
            continue;
        };
        if entries.contains_key(name) {
            entries.insert(name.to_string(), Value::String(new_spec.to_string()));
            return Ok(());
        }
    }
    Err(format!("js: dependency `{name}` not found in package.json"))
}

fn sync_lockfile(package_dir: &Path) -> Result<()> {
    let root = find_package_manager_root(package_dir);
    let manager = detect_package_manager_at(&root);
    let status = match manager {
        PackageManager::Pnpm => Command::new("pnpm").args(["install"]).current_dir(&root).status(),
        PackageManager::Npm => Command::new("npm").args(["install"]).current_dir(&root).status(),
    }
    .map_err(|err| format!("failed to run {} install: {err}", manager_label(manager)))?;
    if !status.success() {
        return Err(format!("{} install failed in {}", manager_label(manager), root.display()));
    }
    Ok(())
}

fn find_package_manager_root(start: &Path) -> PathBuf {
    let mut dir = start.to_path_buf();
    while dir.parent().is_some() {
        if dir.join("pnpm-workspace.yaml").is_file() || dir.join("package-lock.json").is_file() {
            return dir;
        }
        dir = dir.parent().unwrap_or(&dir).to_path_buf();
    }
    start.to_path_buf()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageManager {
    Npm,
    Pnpm,
}

fn detect_package_manager_at(dir: &Path) -> PackageManager {
    if dir.join("pnpm-workspace.yaml").is_file() || dir.join("pnpm-lock.yaml").is_file() {
        return PackageManager::Pnpm;
    }
    PackageManager::Npm
}

fn manager_label(manager: PackageManager) -> &'static str {
    match manager {
        PackageManager::Npm => "npm",
        PackageManager::Pnpm => "pnpm",
    }
}

fn is_registry_spec(spec: &str) -> bool {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    !(lower.starts_with("workspace:")
        || lower.starts_with("link:")
        || lower.starts_with("file:")
        || lower.starts_with("npm:")
        || lower.starts_with("git")
        || lower.starts_with("http:")
        || lower.starts_with("github:")
        || lower.starts_with("patch:"))
}

fn discover_pnpm_workspace_packages(root: &Path) -> Result<Vec<PathBuf>> {
    let yaml_path = root.join("pnpm-workspace.yaml");
    let raw = fs::read_to_string(&yaml_path).map_err(|err| err.to_string())?;
    let mut packages = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("- ") {
            let pattern = rest.trim().trim_matches('"').trim_matches('\'');
            if pattern.ends_with("/*") {
                let parent = root.join(pattern.trim_end_matches("/*"));
                if parent.is_dir() {
                    for entry in fs::read_dir(&parent).map_err(|err| err.to_string())? {
                        let entry = entry.map_err(|err| err.to_string())?;
                        if entry.file_type().map_err(|err| err.to_string())?.is_dir()
                            && entry.path().join("package.json").is_file()
                        {
                            packages.push(entry.path());
                        }
                    }
                }
            }
            else if root.join(pattern).join("package.json").is_file() {
                packages.push(root.join(pattern));
            }
        }
    }
    if root.join("package.json").is_file() {
        packages.push(root.to_path_buf());
    }
    packages.sort();
    packages.dedup();
    Ok(packages)
}

#[cfg(test)]
mod tests {
    use super::{is_registry_spec, npm_spec_version};

    #[test]
    fn skips_non_registry_specs() {
        assert!(!is_registry_spec("workspace:*"));
        assert!(!is_registry_spec("link:../foo"));
        assert!(is_registry_spec("^5.8.3"));
    }

    #[test]
    fn strips_npm_range_prefix() {
        assert_eq!(npm_spec_version("^5.8.3"), "5.8.3");
    }
}
