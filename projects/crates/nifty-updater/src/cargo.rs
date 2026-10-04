use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use dialoguer::MultiSelect;
use serde_json::Value;
use toml::Value as TomlValue;

use crate::{
    Result,
    registry::fetch_crate_latest,
    version::{cargo_version_req, is_upgrade_available},
};

const DEP_TABLES: [&str; 4] = ["dependencies", "dev-dependencies", "build-dependencies", "workspace.dependencies"];

/// One planned cargo dependency bump from registry lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoUpgrade {
    pub crate_name: String,
    pub from_version: String,
    pub to_version: String,
    /// Manifest files that declare this crate (deduped).
    pub manifests: Vec<PathBuf>,
}

pub fn plan_cargo_upgrades(root: &Path) -> Result<Vec<CargoUpgrade>> {
    let metadata = load_cargo_metadata(root)?;
    let resolved = resolved_registry_versions(&metadata);
    let manifest_hits = registry_dependency_manifests(&metadata);

    let mut names = BTreeSet::new();
    for (name, source) in manifest_hits.keys() {
        if source == "registry" {
            names.insert(name.clone());
        }
    }

    let mut upgrades = Vec::new();
    for name in names {
        let from_version = match resolved.get(&name) {
            Some(version) => version.clone(),
            None => {
                eprintln!("cargo: skip `{name}` (not resolved in lockfile)");
                continue;
            }
        };
        let to_version = fetch_crate_latest(&name)?;
        if !is_upgrade_available(&from_version, &to_version) {
            continue;
        }
        let manifests = manifest_hits
            .iter()
            .filter_map(|((dep, source), paths)| (dep == &name && source == "registry").then(|| paths.clone()))
            .flatten()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        upgrades.push(CargoUpgrade { crate_name: name, from_version, to_version, manifests });
    }

    upgrades.sort_by(|a, b| a.crate_name.cmp(&b.crate_name));
    Ok(upgrades)
}

pub fn select_upgrades(upgrades: &[CargoUpgrade]) -> Result<Vec<CargoUpgrade>> {
    let labels: Vec<String> =
        upgrades.iter().map(|item| format!("{} {} -> {}", item.crate_name, item.from_version, item.to_version)).collect();

    let defaults = vec![true; labels.len()];
    let picked = MultiSelect::new()
        .with_prompt("Select cargo upgrades")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .map_err(|err| err.to_string())?;

    Ok(picked.into_iter().map(|index| upgrades[index].clone()).collect())
}

pub fn apply_cargo_upgrades(root: &Path, selected: &[CargoUpgrade]) -> Result<()> {
    for upgrade in selected {
        apply_one_cargo_upgrade(root, upgrade)?;
    }
    Ok(())
}

pub fn apply_cargo_upgrades_all(root: &Path) -> Result<()> {
    let upgrades = plan_cargo_upgrades(root)?;
    apply_cargo_upgrades(root, &upgrades)
}

fn apply_one_cargo_upgrade(root: &Path, upgrade: &CargoUpgrade) -> Result<()> {
    let new_req = cargo_version_req(&upgrade.to_version);
    let mut touched = false;
    for manifest in &upgrade.manifests {
        if patch_manifest_dep(manifest, &upgrade.crate_name, &new_req)? {
            touched = true;
        }
    }
    if !touched {
        return Err(format!("cargo: could not patch manifests for `{}`", upgrade.crate_name));
    }

    let status = Command::new("cargo")
        .args(["update", "-p", &upgrade.crate_name])
        .current_dir(root)
        .status()
        .map_err(|err| format!("failed to run cargo update: {err}"))?;
    if !status.success() {
        return Err(format!("cargo update -p {} failed", upgrade.crate_name));
    }
    println!("cargo: upgraded {} {} -> {}", upgrade.crate_name, upgrade.from_version, upgrade.to_version);
    Ok(())
}

fn patch_manifest_dep(manifest: &Path, crate_name: &str, new_req: &str) -> Result<bool> {
    let raw = fs::read_to_string(manifest).map_err(|err| format!("{}: {err}", manifest.display()))?;
    let mut doc = raw.parse::<TomlValue>().map_err(|err| format!("{}: {err}", manifest.display()))?;
    let mut changed = false;
    for table in DEP_TABLES {
        if set_dep_version(&mut doc, table, crate_name, new_req) {
            changed = true;
        }
    }
    if !changed {
        return Ok(false);
    }
    fs::write(manifest, toml::to_string_pretty(&doc).map_err(|err| err.to_string())?)
        .map_err(|err| format!("{}: {err}", manifest.display()))?;
    Ok(true)
}

fn set_dep_version(doc: &mut TomlValue, table: &str, name: &str, new_req: &str) -> bool {
    let root = match doc.as_table_mut() {
        Some(root) => root,
        None => return false,
    };
    let deps = match root.get_mut(table).and_then(TomlValue::as_table_mut) {
        Some(deps) => deps,
        None => return false,
    };
    let entry = match deps.get_mut(name) {
        Some(entry) => entry,
        None => return false,
    };
    match entry {
        TomlValue::String(_) => {
            *entry = TomlValue::String(new_req.to_string());
            true
        }
        TomlValue::Table(table) => {
            table.insert("version".to_string(), TomlValue::String(new_req.to_string()));
            true
        }
        _ => false,
    }
}

fn load_cargo_metadata(root: &Path) -> Result<Value> {
    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(format!("cargo: missing {}", manifest.display()));
    }
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--manifest-path"])
        .arg(&manifest)
        .output()
        .map_err(|err| format!("failed to run cargo metadata: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("cargo metadata failed:\n{stdout}{stderr}"));
    }
    serde_json::from_slice(&output.stdout).map_err(|err| format!("cargo metadata JSON: {err}"))
}

fn resolved_registry_versions(metadata: &Value) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let packages = metadata.get("packages").and_then(Value::as_array);
    for pkg in packages.into_iter().flatten() {
        let source = pkg.get("source").and_then(Value::as_str);
        if !source.is_some_and(|s| s.starts_with("registry")) {
            continue;
        }
        let name = pkg.get("name").and_then(Value::as_str);
        let version = pkg.get("version").and_then(Value::as_str);
        if let (Some(name), Some(version)) = (name, version) {
            out.insert(name.to_string(), version.to_string());
        }
    }
    out
}

fn registry_dependency_manifests(metadata: &Value) -> BTreeMap<(String, String), BTreeSet<PathBuf>> {
    let mut out: BTreeMap<(String, String), BTreeSet<PathBuf>> = BTreeMap::new();
    let packages = metadata.get("packages").and_then(Value::as_array);
    for pkg in packages.into_iter().flatten() {
        let manifest = pkg.get("manifest_path").and_then(Value::as_str).map(PathBuf::from);
        let deps = pkg.get("dependencies").and_then(Value::as_array);
        for dep in deps.into_iter().flatten() {
            let name = dep.get("name").and_then(Value::as_str);
            let source = dep
                .get("source")
                .and_then(Value::as_str)
                .map(|s| if s.starts_with("registry") { "registry" } else { "other" })
                .unwrap_or("other");
            if source != "registry" {
                continue;
            }
            if let (Some(name), Some(manifest)) = (name, manifest.clone()) {
                out.entry((name.to_string(), source.to_string())).or_default().insert(manifest);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{patch_manifest_dep, set_dep_version};
    use toml::Value as TomlValue;

    #[test]
    fn patches_inline_and_string_dependency_tables() {
        let mut doc = r#"
[dependencies]
serde = "1.0"
gix = { version = "0.68", features = ["revision"] }
"#
        .parse::<TomlValue>()
        .expect("toml");
        assert!(set_dep_version(&mut doc, "dependencies", "serde", "^1.0.220"));
        assert!(set_dep_version(&mut doc, "dependencies", "gix", "^0.69.0"));
        let dir = std::env::temp_dir().join(format!("nifty-updater-cargo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let manifest = dir.join("Cargo.toml");
        std::fs::write(&manifest, toml::to_string_pretty(&doc).expect("pretty")).expect("write");
        assert!(patch_manifest_dep(&manifest, "serde", "^1.0.220").expect("patch"));
        let next = std::fs::read_to_string(&manifest).expect("read");
        assert!(next.contains("^1.0.220"));
        std::fs::remove_dir_all(dir).ok();
    }
}
