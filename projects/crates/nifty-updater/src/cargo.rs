use std::path::Path;
use std::process::Command;

use dialoguer::MultiSelect;

use crate::Result;

/// One planned cargo dependency bump from `cargo upgrade --dry-run`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CargoUpgrade {
    pub crate_name: String,
    pub from_version: String,
    pub to_version: String,
    pub line: String,
}

pub fn plan_cargo_upgrades(root: &Path) -> Result<Vec<CargoUpgrade>> {
    let output = Command::new("cargo")
        .args(["upgrade", "--workspace", "--dry-run"])
        .current_dir(root)
        .output()
        .map_err(|err| format!("failed to run cargo upgrade: {err}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    if !output.status.success() && combined.contains("no upgrades found") {
        return Ok(Vec::new());
    }

    let upgrades = parse_upgrade_lines(&combined);
    if !output.status.success() && upgrades.is_empty() {
        return Err(format!("cargo upgrade --dry-run failed:\n{combined}"));
    }

    Ok(upgrades)
}

pub fn select_upgrades(upgrades: &[CargoUpgrade]) -> Result<Vec<CargoUpgrade>> {
    let labels: Vec<String> = upgrades
        .iter()
        .map(|item| format!("{} {} -> {}", item.crate_name, item.from_version, item.to_version))
        .collect();

    let defaults: Vec<bool> = vec![true; labels.len()];
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
        let status = Command::new("cargo")
            .args(["upgrade", "--workspace", "-p", &upgrade.crate_name])
            .current_dir(root)
            .status()
            .map_err(|err| format!("failed to run cargo upgrade: {err}"))?;
        if !status.success() {
            return Err(format!("cargo upgrade failed for {}", upgrade.crate_name));
        }
        println!("cargo: upgraded {} -> {}", upgrade.crate_name, upgrade.to_version);
    }
    Ok(())
}

pub fn apply_cargo_upgrades_all(root: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args(["upgrade", "--workspace"])
        .current_dir(root)
        .status()
        .map_err(|err| format!("failed to run cargo upgrade: {err}"))?;
    if !status.success() {
        return Err("cargo upgrade --workspace failed".to_string());
    }
    println!("cargo: upgraded workspace dependencies");
    Ok(())
}

fn parse_upgrade_lines(text: &str) -> Vec<CargoUpgrade> {
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(upgrade) = parse_upgrade_line(line) {
            out.push(upgrade);
        }
    }
    out
}

fn parse_upgrade_line(line: &str) -> Option<CargoUpgrade> {
    let trimmed = line.trim();
    if !trimmed.contains("->") {
        return None;
    }

    // cargo-edit: `serde v1.0.197 -> v1.0.204`
    let (left, right) = trimmed.split_once("->")?;
    let left = left.trim();
    let to_version = right.trim().trim_start_matches('v').trim().to_string();

    let (crate_name, from_version) = parse_name_version(left)?;
    Some(CargoUpgrade {
        crate_name,
        from_version,
        to_version,
        line: trimmed.to_string(),
    })
}

fn parse_name_version(left: &str) -> Option<(String, String)> {
    let left = left.trim();
    if let Some((name, version)) = left.rsplit_once(' ') {
        return Some((
            name.trim().to_string(),
            version.trim().trim_start_matches('v').trim().to_string(),
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_upgrade_line;

    #[test]
    fn parses_cargo_edit_dry_run_line() {
        let parsed = parse_upgrade_line("  serde v1.0.197 -> v1.0.204").expect("upgrade");
        assert_eq!(parsed.crate_name, "serde");
        assert_eq!(parsed.from_version, "1.0.197");
        assert_eq!(parsed.to_version, "1.0.204");
    }
}
