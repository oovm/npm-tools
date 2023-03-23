use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use nifty_config::detect_project_layout;
use serde::Deserialize;
use serde_json::Value;

use crate::Result;

const DEPENDENCY_FIELDS: [&str; 4] = [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

/// Parsed `package.json` fields used for publishing.
#[derive(Debug, Clone, Deserialize)]
pub struct PackageManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub dev_dependencies: BTreeMap<String, String>,
    #[serde(rename = "optionalDependencies", default)]
    pub optional_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub peer_dependencies: BTreeMap<String, String>,
}

/// One npm package directory in the workspace.
#[derive(Debug, Clone)]
pub struct NpmPackage {
    pub name: String,
    pub version: String,
    pub dir: PathBuf,
    pub manifest_path: PathBuf,
    pub private: bool,
    pub manifest: PackageManifest,
}

pub fn find_workspace_root(cwd: &Path) -> Result<PathBuf> {
    let layout = detect_project_layout(cwd);
    let candidates = [layout.root.clone(), cwd.to_path_buf()];
    for candidate in candidates {
        let crates = candidate.join("projects").join("crates");
        let packages = candidate.join("projects").join("packages");
        if crates.is_dir() && packages.is_dir() {
            return Ok(candidate);
        }
        if candidate.join("pnpm-workspace.yaml").is_file() {
            return Ok(candidate);
        }
    }
    Err("could not find workspace root (Nifty hybrid or pnpm-workspace.yaml)".into())
}

pub fn list_workspace_packages(root: &Path) -> Result<Vec<NpmPackage>> {
    let nifty_packages = root.join("projects").join("packages");
    if nifty_packages.is_dir() {
        return list_packages_in_dir(&nifty_packages);
    }
    list_pnpm_workspace_packages(root)
}

fn list_packages_in_dir(packages_dir: &Path) -> Result<Vec<NpmPackage>> {
    let mut packages = Vec::new();
    for entry in fs::read_dir(packages_dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        if !entry.file_type().map_err(|err| err.to_string())?.is_dir() {
            continue;
        }
        if entry.file_name() == "node_modules" {
            continue;
        }
        let manifest_path = entry.path().join("package.json");
        if !manifest_path.is_file() {
            continue;
        }
        packages.push(load_package(&entry.path(), manifest_path)?);
    }
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(packages)
}

fn list_pnpm_workspace_packages(root: &Path) -> Result<Vec<NpmPackage>> {
    let yaml_path = root.join("pnpm-workspace.yaml");
    let text = fs::read_to_string(&yaml_path).map_err(|err| err.to_string())?;
    let patterns = parse_pnpm_workspace_patterns(&text);
    if patterns.is_empty() {
        return Err(format!("no packages globs in {}", yaml_path.display()));
    }

    let mut seen = HashSet::new();
    let mut packages = Vec::new();
    for pattern in patterns {
        for dir in expand_workspace_glob(root, &pattern)? {
            let manifest_path = dir.join("package.json");
            if !manifest_path.is_file() {
                continue;
            }
            let package = load_package(&dir, manifest_path)?;
            if package.private || !seen.insert(package.name.clone()) {
                continue;
            }
            packages.push(package);
        }
    }
    packages.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(packages)
}

fn parse_pnpm_workspace_patterns(text: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    let mut in_packages = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("packages:") {
            in_packages = true;
            continue;
        }
        if !in_packages {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !trimmed.starts_with('-') {
            if patterns.is_empty() {
                continue;
            }
            break;
        }
        let value = trimmed
            .trim_start_matches('-')
            .trim()
            .trim_matches('\'')
            .trim_matches('"');
        if !value.is_empty() {
            patterns.push(value.to_string());
        }
    }
    patterns
}

fn expand_workspace_glob(root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let normalized = pattern.replace('\\', "/");
    if normalized.ends_with("/*") {
        let base = root.join(normalized.trim_end_matches("/*"));
        if !base.is_dir() {
            return Ok(Vec::new());
        }
        let mut dirs = Vec::new();
        for entry in fs::read_dir(&base).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            if entry.file_type().map_err(|err| err.to_string())?.is_dir() {
                dirs.push(entry.path());
            }
        }
        return Ok(dirs);
    }
    let path = root.join(normalized);
    if path.is_dir() {
        return Ok(vec![path]);
    }
    Ok(Vec::new())
}

pub fn load_package(dir: &Path, manifest_path: PathBuf) -> Result<NpmPackage> {
    let raw = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
    let manifest: PackageManifest = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    if manifest.name.is_empty() || manifest.version.is_empty() {
        return Err(format!("package.json missing name or version: {}", manifest_path.display()));
    }
    Ok(NpmPackage {
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        dir: dir.to_path_buf(),
        manifest_path,
        private: manifest.private,
        manifest,
    })
}

pub fn collect_internal_dependency_names(
    package: &NpmPackage,
    by_name: &BTreeMap<String, NpmPackage>,
) -> HashSet<String> {
    let mut deps = HashSet::new();
    for field in DEPENDENCY_FIELDS {
        let entries = dependency_entries(package, field);
        for (dep_name, spec) in entries {
            if by_name.contains_key(&dep_name) {
                deps.insert(dep_name);
                continue;
            }
            if let Some(resolved) = resolve_workspace_dependency_name(&spec, &package.dir, by_name) {
                deps.insert(resolved);
            }
        }
    }
    deps
}

fn dependency_entries(package: &NpmPackage, field: &str) -> Vec<(String, String)> {
    let map = match field {
        "dependencies" => &package.manifest.dependencies,
        "devDependencies" => &package.manifest.dev_dependencies,
        "optionalDependencies" => &package.manifest.optional_dependencies,
        "peerDependencies" => &package.manifest.peer_dependencies,
        _ => return Vec::new(),
    };
    map.iter().map(|(name, spec)| (name.clone(), spec.clone())).collect()
}

fn resolve_workspace_dependency_name(
    spec: &str,
    package_dir: &Path,
    by_name: &BTreeMap<String, NpmPackage>,
) -> Option<String> {
    if spec.starts_with("workspace:") {
        return None;
    }
    if !spec.starts_with("file:") {
        return None;
    }
    let target_dir = package_dir.join(spec.trim_start_matches("file:"));
    for candidate in by_name.values() {
        if candidate.dir == target_dir {
            return Some(candidate.name.clone());
        }
    }
    let manifest_path = target_dir.join("package.json");
    if !manifest_path.is_file() {
        return None;
    }
    let raw = fs::read_to_string(&manifest_path).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    value.get("name")?.as_str().map(str::to_string)
}
