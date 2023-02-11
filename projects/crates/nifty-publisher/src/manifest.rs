use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use crate::workspace::NpmPackage;
use crate::Result;

const DEPENDENCY_FIELDS: [&str; 4] = [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
];

pub fn patch_manifest_for_publish(
    original: &str,
    package: &NpmPackage,
    by_name: &BTreeMap<String, NpmPackage>,
) -> Result<String> {
    let mut value: Value = serde_json::from_str(original).map_err(|err| err.to_string())?;
    let object = value.as_object_mut().ok_or_else(|| "package.json root must be an object".to_string())?;

    for field in DEPENDENCY_FIELDS {
        let Some(entries) = object.get(field).and_then(Value::as_object) else {
            continue;
        };
        let patched = patch_dependency_entries(entries, &package.dir, by_name)?;
        object.insert(field.to_string(), Value::Object(patched));
    }

    Ok(format!("{}\n", serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?))
}

fn patch_dependency_entries(
    entries: &serde_json::Map<String, Value>,
    package_dir: &Path,
    by_name: &BTreeMap<String, NpmPackage>,
) -> Result<serde_json::Map<String, Value>> {
    let mut next = serde_json::Map::new();
    for (name, spec_value) in entries {
        let spec = spec_value.as_str().ok_or_else(|| format!("dependency {name} must be a string"))?;
        let version = resolve_workspace_dependency(name, spec, package_dir, by_name)
            .map(|package| package.version.clone())
            .unwrap_or_else(|| spec.to_string());
        next.insert(name.clone(), Value::String(version));
    }
    Ok(next)
}

fn resolve_workspace_dependency<'a>(
    name: &str,
    spec: &str,
    package_dir: &Path,
    by_name: &'a BTreeMap<String, NpmPackage>,
) -> Option<&'a NpmPackage> {
    if let Some(direct) = by_name.get(name) {
        if spec.starts_with("file:") || spec.starts_with("workspace:") {
            return Some(direct);
        }
    }
    if !spec.starts_with("file:") {
        return None;
    }
    let target_dir = package_dir.join(spec.trim_start_matches("file:"));
    by_name.values().find(|candidate| candidate.dir == target_dir)
}
