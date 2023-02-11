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
    }
    Err("could not find Nifty workspace (expected projects/crates and projects/packages)".into())
}

pub fn list_workspace_packages(root: &Path) -> Result<Vec<NpmPackage>> {
    let packages_dir = root.join("projects").join("packages");
    if !packages_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut packages = Vec::new();
    for entry in fs::read_dir(&packages_dir).map_err(|err| err.to_string())? {
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
