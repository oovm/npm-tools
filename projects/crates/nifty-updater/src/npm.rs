use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use dialoguer::MultiSelect;
use nifty_config::ProjectLayout;
use serde_json::Value;

use crate::Result;

/// One outdated npm dependency from `npm outdated --json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmOutdated {
    pub name: String,
    pub current: String,
    pub wanted: String,
    pub latest: String,
}

pub fn discover_package_dirs(layout: &ProjectLayout) -> Result<Vec<PathBuf>> {
    let mut roots = BTreeSet::new();

    if let Some(root) = &layout.npm_workspace_root {
        roots.insert(root.clone());
    }

    if let Some(manifest) = &layout.package_manifest {
        if let Some(parent) = manifest.parent() {
            roots.insert(parent.to_path_buf());
        }
    }

    if layout.kind == nifty_config::ProjectKind::Hybrid {
        let packages_dir = layout.root.join("projects").join("packages");
        if packages_dir.is_dir() {
            for entry in fs::read_dir(&packages_dir).map_err(|err| err.to_string())? {
                let entry = entry.map_err(|err| err.to_string())?;
                if entry.file_type().map_err(|err| err.to_string())?.is_dir() {
                    let package_json = entry.path().join("package.json");
                    if package_json.is_file() {
                        roots.insert(entry.path());
                    }
                }
            }
        }
    }

    Ok(roots.into_iter().collect())
}

pub fn list_outdated(package_dir: &Path) -> Result<Vec<NpmOutdated>> {
    let output = Command::new("npm")
        .args(["outdated", "--json"])
        .current_dir(package_dir)
        .output()
        .map_err(|err| format!("failed to run npm outdated: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parsed: Value = serde_json::from_str(&stdout).map_err(|err| err.to_string())?;
    let Some(object) = parsed.as_object() else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for (name, entry) in object {
        if let Some(item) = parse_outdated_entry(name, entry) {
            out.push(item);
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
    let prompt = format!("Select npm upgrades ({})", package_dir.display());
    let picked = MultiSelect::new()
        .with_prompt(prompt)
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .map_err(|err| err.to_string())?;

    Ok(picked.into_iter().map(|index| outdated[index].clone()).collect())
}

pub fn apply_npm_upgrades(package_dir: &Path, selected: &[NpmOutdated]) -> Result<()> {
    for item in selected {
        let spec = format!("{}@{}", item.name, item.latest);
        let status = Command::new("npm")
            .args(["install", &spec])
            .current_dir(package_dir)
            .status()
            .map_err(|err| format!("failed to run npm install: {err}"))?;
        if !status.success() {
            return Err(format!("npm install failed for {}", item.name));
        }
        println!("npm ({}): upgraded {} -> {}", package_dir.display(), item.name, item.latest);
    }
    Ok(())
}

pub fn apply_npm_update_all(package_dir: &Path) -> Result<()> {
    let status = Command::new("npm")
        .args(["update"])
        .current_dir(package_dir)
        .status()
        .map_err(|err| format!("failed to run npm update: {err}"))?;
    if !status.success() {
        return Err(format!("npm update failed in {}", package_dir.display()));
    }
    println!("npm ({}): updated dependencies", package_dir.display());
    Ok(())
}

fn parse_outdated_entry(name: &str, entry: &Value) -> Option<NpmOutdated> {
    let current = entry.get("current")?.as_str()?.to_string();
    let wanted = entry.get("wanted")?.as_str()?.to_string();
    let latest = entry.get("latest")?.as_str()?.to_string();
    Some(NpmOutdated {
        name: name.to_string(),
        current,
        wanted,
        latest,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_outdated_entry;
    use serde_json::json;

    #[test]
    fn parses_npm_outdated_entry() {
        let item = parse_outdated_entry(
            "typescript",
            &json!({
                "current": "5.8.3",
                "wanted": "5.8.3",
                "latest": "5.9.2"
            }),
        )
        .expect("entry");
        assert_eq!(item.name, "typescript");
        assert_eq!(item.latest, "5.9.2");
    }
}
