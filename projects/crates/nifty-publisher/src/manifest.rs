use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use serde_json::Value;

use crate::{Result, workspace::NpmPackage};

const DEPENDENCY_FIELDS: [&str; 4] = ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"];

/// Options for [`patch_manifest_for_publish`].
#[derive(Debug, Clone, Copy, Default)]
pub struct PatchPublishOptions<'a> {
    /// Force `version` on the published manifest and on workspace dependency specs
    /// (e.g. `0.0.0` for `--placeholder`).
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
        let patched =
            patch_dependency_entries(entries, &package.dir, by_name, opts.publish_version, opts.placeholder_dep_names)?;
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
    // Never synthesize `repository`: npm provenance compares it with the GitHub
    // repository running the workflow, so a fabricated URL fails the publish with
    // E422. Workspaces that publish with provenance must declare their own repo.
    let publish_config = object.entry("publishConfig".to_string()).or_insert_with(|| Value::Object(serde_json::Map::new()));
    if let Value::Object(map) = publish_config {
        map.entry("access".to_string()).or_insert_with(|| Value::String("public".to_string()));
    }
}

fn patch_dependency_entries(
    entries: &serde_json::Map<String, Value>,
    package_dir: &Path,
    by_name: &BTreeMap<String, NpmPackage>,
    publish_version: Option<&str>,
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
                publish_version.unwrap_or(package.version.as_str()).to_string()
            }
        }
        else {
            spec.to_string()
        };
        next.insert(name.clone(), Value::String(version));
    }
    Ok(next)
}

/// Reject exact internal dependency pins that publish would ship pointing at an older version.
///
/// Publish rewrites `workspace:` and `file:` specs to the released version. Exact pins and
/// ranges are left untouched, so a package that pins an internal dependency must then carry
/// the version the dependency is published as. Failing here keeps a stale pin from silently
/// shipping a package that resolves to the previous release.
pub fn verify_publish_dependency_pins(
    package: &NpmPackage,
    by_name: &BTreeMap<String, NpmPackage>,
    publish_plan: &BTreeSet<String>,
    publish_version: Option<&str>,
) -> Result<()> {
    let fields = [
        ("dependencies", &package.manifest.dependencies),
        ("optionalDependencies", &package.manifest.optional_dependencies),
        ("peerDependencies", &package.manifest.peer_dependencies),
    ];
    for (field, entries) in fields {
        for (name, spec) in entries {
            if !is_exact_version(spec) {
                continue;
            }
            let Some(target) = by_name.get(name)
            else {
                continue;
            };
            // Only a dependency that is part of this publish takes the `--version` override.
            let expected = if publish_plan.contains(name) {
                publish_version.unwrap_or(target.version.as_str())
            }
            else {
                target.version.as_str()
            };
            if spec != expected {
                return Err(format!(
                    "`{}` declares `{name}` as the exact pin `{spec}` in `{field}`, but publish ships `{name}` as {expected}. Exact pins are not rewritten — use `workspace:*` for internal dependencies",
                    package.name
                ));
            }
        }
    }
    Ok(())
}

fn is_exact_version(spec: &str) -> bool {
    let core = spec.split(['-', '+']).next().unwrap_or(spec);
    let mut parts = core.split('.');
    let (Some(major), Some(minor), Some(patch), None) = (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    [major, minor, patch].iter().all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
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

    use super::{PatchPublishOptions, patch_manifest_for_publish, verify_publish_dependency_pins};
    use crate::workspace::{NpmPackage, PackageManifest};

    fn from_manifest(json: &str) -> NpmPackage {
        let manifest: PackageManifest = serde_json::from_str(json).expect("manifest json");
        let name = manifest.name.clone();
        let version = manifest.version.clone();
        NpmPackage {
            dir: PathBuf::from(&name),
            manifest_path: PathBuf::from(&name).join("package.json"),
            private: manifest.private,
            name,
            version,
            manifest,
        }
    }

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
            main: None,
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

    #[test]
    fn version_override_applies_to_workspace_dependencies() {
        let main = pkg("@scope/main", "0.1.0");
        let platform = pkg("@scope/platform", "0.1.0");
        let by_name = BTreeMap::from([("@scope/main".to_string(), main.clone()), ("@scope/platform".to_string(), platform)]);
        let original = r#"{
  "name": "@scope/main",
  "version": "0.1.0",
  "dependencies": {
    "@scope/platform": "workspace:*"
  }
}"#;
        let patched = patch_manifest_for_publish(
            original,
            &main,
            &by_name,
            PatchPublishOptions { publish_version: Some("0.3.1"), placeholder_dep_names: None },
        )
        .expect("patch");
        let value: serde_json::Value = serde_json::from_str(&patched).expect("json");
        assert_eq!(value["version"], "0.3.1");
        assert_eq!(value["dependencies"]["@scope/platform"], "0.3.1");
    }

    #[test]
    fn publish_manifest_never_invents_a_repository() {
        let main = pkg("@vmz/vmz", "0.3.4");
        let by_name = BTreeMap::from([("@vmz/vmz".to_string(), main.clone())]);
        let original = r#"{
  "name": "@vmz/vmz",
  "version": "0.1.0"
}"#;
        let patched = patch_manifest_for_publish(original, &main, &by_name, PatchPublishOptions::default()).expect("patch");
        let value: serde_json::Value = serde_json::from_str(&patched).expect("json");
        assert!(value.get("repository").is_none(), "{patched}");
        assert_eq!(value["publishConfig"]["access"], "public");
    }

    #[test]
    fn stale_exact_pin_blocks_publish() {
        let main = from_manifest(
            r#"{
  "name": "@scope/main",
  "version": "0.2.0",
  "dependencies": {
    "@scope/platform": "0.1.4"
  }
}"#,
        );
        let platform = pkg("@scope/platform", "0.2.0");
        let by_name = BTreeMap::from([("@scope/main".to_string(), main.clone()), ("@scope/platform".to_string(), platform)]);
        let plan = BTreeSet::from(["@scope/main".to_string(), "@scope/platform".to_string()]);
        let error = verify_publish_dependency_pins(&main, &by_name, &plan, None).expect_err("stale pin must fail");
        assert!(error.contains("@scope/platform"), "{error}");
        assert!(error.contains("0.1.4"), "{error}");
        assert!(error.contains("0.2.0"), "{error}");
        assert!(error.contains("workspace:*"), "{error}");
    }

    #[test]
    fn matching_pins_workspace_specs_and_ranges_pass() {
        let main = from_manifest(
            r#"{
  "name": "@scope/main",
  "version": "0.2.0",
  "dependencies": {
    "@scope/platform": "0.2.0"
  },
  "optionalDependencies": {
    "@scope/platform": "workspace:*"
  },
  "peerDependencies": {
    "@scope/platform": "^0.1.4"
  }
}"#,
        );
        let platform = pkg("@scope/platform", "0.2.0");
        let by_name = BTreeMap::from([("@scope/main".to_string(), main.clone()), ("@scope/platform".to_string(), platform)]);
        let plan = BTreeSet::from(["@scope/main".to_string(), "@scope/platform".to_string()]);
        verify_publish_dependency_pins(&main, &by_name, &plan, None)
            .expect("matching pins and workspace/range specs are allowed");
    }

    #[test]
    fn publish_version_override_makes_a_manifest_pin_stale() {
        let main = from_manifest(r#"{"name":"@scope/main","version":"0.2.0","dependencies":{"@scope/platform":"0.2.0"}}"#);
        let platform = pkg("@scope/platform", "0.2.0");
        let by_name = BTreeMap::from([("@scope/main".to_string(), main.clone()), ("@scope/platform".to_string(), platform)]);
        let plan = BTreeSet::from(["@scope/main".to_string(), "@scope/platform".to_string()]);
        let error =
            verify_publish_dependency_pins(&main, &by_name, &plan, Some("0.2.1")).expect_err("override makes the pin stale");
        assert!(error.contains("0.2.1"), "{error}");
    }

    #[test]
    fn publish_version_override_spares_dependencies_outside_the_plan() {
        let main = from_manifest(r#"{"name":"@scope/main","version":"0.2.0","dependencies":{"@scope/platform":"0.2.0"}}"#);
        let platform = pkg("@scope/platform", "0.2.0");
        let by_name = BTreeMap::from([("@scope/main".to_string(), main.clone()), ("@scope/platform".to_string(), platform)]);
        let plan = BTreeSet::from(["@scope/main".to_string()]);
        verify_publish_dependency_pins(&main, &by_name, &plan, Some("0.3.0"))
            .expect("a dependency outside the plan keeps its own workspace version");
    }

    #[test]
    fn pins_outside_the_workspace_are_ignored() {
        let main = from_manifest(r#"{"name":"@scope/main","version":"0.2.0","dependencies":{"react":"18.2.0"}}"#);
        let by_name = BTreeMap::from([("@scope/main".to_string(), main.clone())]);
        let plan = BTreeSet::from(["@scope/main".to_string()]);
        verify_publish_dependency_pins(&main, &by_name, &plan, None).expect("registry dependencies keep their pin");
    }
}
