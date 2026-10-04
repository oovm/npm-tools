use std::{collections::BTreeMap, path::Path};

use serde_json::Value;

use crate::{Result, workspace::NpmPackage};

const PUBLISH_REPOSITORY_URL: &str = "git+https://github.com/oovm/npm-tools.git";

const DEPENDENCY_FIELDS: [&str; 4] = ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"];

/// Options for [`patch_manifest_for_publish`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PatchPublishOptions<'a> {
    /// Force `version` on the published manifest (e.g. `0.0.0` for `--placeholder`).
    pub publish_version: Option<&'a str>,
    /// Workspace dependency names that should resolve to `0.0.0` this run.
    pub placeholder_dep_names: Option<&'a std::collections::BTreeSet<String>>,
}

pub fn patch_manifest_for_publish(
    original: &str,
    package: &NpmPackage,
    by_name: &BTreeMap<String, NpmPackage>,
    opts: PatchPublishOptions<'_>,
) -> Result<String> {
    let mut value: Value = serde_json::from_str(original).map_err(|err| err.to_string())?;
    let object = value.as_object_mut().ok_or_else(|| "package.json root must be an object".to_string())?;

    if let Some(version) = opts.publish_version {
        object.insert("version".to_string(), Value::String(version.to_string()));
    }

    for field in DEPENDENCY_FIELDS {
        let Some(entries) = object.get(field).and_then(Value::as_object)
        else {
            continue;
        };
        let patched = patch_dependency_entries(entries, &package.dir, by_name, opts.placeholder_dep_names)?;
        object.insert(field.to_string(), Value::Object(patched));
    }

    object.remove("private");
    if let Some(registry_name) =
        package.manifest.publish_config.as_ref().and_then(|config| config.name.as_ref()).filter(|name| !name.is_empty())
    {
        object.insert("name".to_string(), Value::String(registry_name.clone()));
    }
    ensure_publish_metadata(object);

    Ok(format!("{}\n", serde_json::to_string_pretty(&value).map_err(|err| err.to_string())?))
}

fn ensure_publish_metadata(object: &mut serde_json::Map<String, Value>) {
    let has_repo_url = object
        .get("repository")
        .and_then(Value::as_object)
        .and_then(|repo| repo.get("url"))
        .and_then(Value::as_str)
        .is_some_and(|url| !url.is_empty());
    if !has_repo_url {
        object.insert(
            "repository".to_string(),
            serde_json::json!({
                "type": "git",
                "url": PUBLISH_REPOSITORY_URL
            }),
        );
    }

    let publish_config = object.entry("publishConfig".to_string()).or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Value::Object(map) = publish_config {
        map.entry("access".to_string()).or_insert_with(|| Value::String("public".to_string()));
    }
}

fn patch_dependency_entries(
    entries: &serde_json::Map<String, Value>,
    package_dir: &Path,
    by_name: &BTreeMap<String, NpmPackage>,
    placeholder_dep_names: Option<&std::collections::BTreeSet<String>>,
) -> Result<serde_json::Map<String, Value>> {
    let mut next = serde_json::Map::new();
    for (name, spec_value) in entries {
        let spec = spec_value.as_str().ok_or_else(|| format!("dependency {name} must be a string"))?;
        let version = if let Some(package) = resolve_workspace_dependency(name, spec, package_dir, by_name) {
            if placeholder_dep_names.is_some_and(|names| names.contains(name)) {
                "0.0.0".to_string()
            }
            else {
                package.version.clone()
            }
        }
        else {
            spec.to_string()
        };
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

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::PathBuf,
    };

    use super::{PatchPublishOptions, patch_manifest_for_publish};
    use crate::workspace::{NpmPackage, PackageManifest};

    fn pkg(name: &str, version: &str) -> NpmPackage {
        let manifest = PackageManifest {
            name: name.to_string(),
            version: version.to_string(),
            private: false,
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            os: None,
            cpu: None,
            publish_config: None,
        };
        NpmPackage {
            name: name.to_string(),
            version: version.to_string(),
            dir: PathBuf::from(name),
            manifest_path: PathBuf::from(name).join("package.json"),
            private: false,
            manifest,
        }
    }

    #[test]
    fn placeholder_forces_version_and_workspace_deps() {
        let main = pkg("@scope/main", "0.1.5");
        let platform = pkg("@scope/platform", "0.1.5");
        let by_name = BTreeMap::from([("@scope/main".to_string(), main.clone()), ("@scope/platform".to_string(), platform)]);
        let placeholder_deps = BTreeSet::from(["@scope/platform".to_string()]);
        let original = r#"{
  "name": "@scope/main",
  "version": "0.1.5",
  "dependencies": {
    "@scope/platform": "workspace:*"
  }
}"#;
        let patched = patch_manifest_for_publish(
            original,
            &main,
            &by_name,
            PatchPublishOptions { publish_version: Some("0.0.0"), placeholder_dep_names: Some(&placeholder_deps) },
        )
        .expect("patch");
        let value: serde_json::Value = serde_json::from_str(&patched).expect("json");
        assert_eq!(value["version"], "0.0.0");
        assert_eq!(value["dependencies"]["@scope/platform"], "0.0.0");
    }
}
