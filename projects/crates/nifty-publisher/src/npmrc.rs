use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Result;

const DEFAULT_REGISTRY: &str = "https://registry.npmjs.org";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NpmRc {
    default_registry: String,
    scoped_registries: BTreeMap<String, String>,
    auth_tokens: BTreeMap<String, String>,
}

impl NpmRc {
    pub fn load_merged(workspace_root: &Path) -> Self {
        let mut merged = Self::empty();
        if let Some(path) = user_npmrc_path() {
            merge_file(&mut merged, &path);
        }
        merge_file(&mut merged, &workspace_root.join(".npmrc"));
        merged
    }

    pub fn empty() -> Self {
        Self {
            default_registry: DEFAULT_REGISTRY.to_string(),
            scoped_registries: BTreeMap::new(),
            auth_tokens: BTreeMap::new(),
        }
    }

    pub fn from_text(text: &str) -> Self {
        let mut config = Self::empty();
        config.apply_text(text);
        config
    }

    pub fn merge(&mut self, other: Self) {
        if other.default_registry != DEFAULT_REGISTRY {
            self.default_registry = other.default_registry;
        }
        self.scoped_registries.extend(other.scoped_registries);
        self.auth_tokens.extend(other.auth_tokens);
    }

    pub fn registry_for_package(&self, package: &str) -> String {
        if let Some(scope) = package_scope(package) {
            if let Some(registry) = self.scoped_registries.get(scope) {
                return registry.clone();
            }
        }
        self.default_registry.clone()
    }

    pub fn token_for_registry(&self, registry: &str) -> Option<String> {
        let host = normalize_registry_host(registry);
        self.auth_tokens.get(&host).cloned()
    }

    pub fn token_for_package(&self, package: &str) -> Option<String> {
        self.token_for_registry(&self.registry_for_package(package))
    }

    pub fn default_token(&self) -> Option<String> {
        self.token_for_registry(&self.default_registry)
    }

    fn apply_text(&mut self, text: &str) {
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            let Some((key, value)) = split_kv(line) else {
                continue;
            };
            if key.starts_with("//") && key.ends_with(":_authToken") {
                let host = key
                    .trim_start_matches("//")
                    .trim_end_matches(":_authToken")
                    .trim_end_matches('/');
                if !host.is_empty() && !value.is_empty() {
                    self.auth_tokens
                        .insert(normalize_registry_host(host), value.to_string());
                }
                continue;
            }
            if key == "registry" && !value.is_empty() {
                self.default_registry = normalize_registry_url(value);
                continue;
            }
            if key.starts_with('@') && key.ends_with(":registry") && !value.is_empty() {
                let scope = key.trim_end_matches(":registry");
                self.scoped_registries
                    .insert(scope.to_string(), normalize_registry_url(value));
            }
        }
    }
}

pub fn user_npmrc_path() -> Option<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()?;
    Some(Path::new(&home).join(".npmrc"))
}

pub fn npm_config_path(workspace_root: &Path) -> Option<PathBuf> {
    let project = workspace_root.join(".npmrc");
    if project.is_file() {
        return Some(project);
    }
    user_npmrc_path().filter(|path| path.is_file())
}

fn merge_file(target: &mut NpmRc, path: &Path) {
    if !path.is_file() {
        return;
    }
    if let Ok(text) = fs::read_to_string(path) {
        target.merge(NpmRc::from_text(&text));
    }
}

fn split_kv(line: &str) -> Option<(&str, &str)> {
    let mut key = line;
    if key.starts_with("export ") {
        key = key.trim_start_matches("export ").trim_start();
    }
    let Some((key, value)) = key.split_once('=') else {
        return None;
    };
    Some((key.trim(), unquote(value.trim())))
}

fn unquote(value: &str) -> &str {
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        return value
            .get(1..value.len().saturating_sub(1))
            .unwrap_or(value);
    }
    value
}

fn package_scope(name: &str) -> Option<&str> {
    if !name.starts_with('@') {
        return None;
    }
    name.split('/').next()
}

pub fn normalize_registry_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return DEFAULT_REGISTRY.to_string();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.trim_end_matches('/').to_string()
    } else {
        format!("https://{}", trimmed.trim_end_matches('/'))
    }
}

pub fn normalize_registry_host(value: &str) -> String {
    let url = normalize_registry_url(value);
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_npmrc_auth_and_registry() {
        let config = NpmRc::from_text(
            "registry=https://registry.npmjs.org/\n//registry.npmjs.org/:_authToken=npm_test_token\n",
        );
        assert_eq!(config.default_registry, "https://registry.npmjs.org");
        assert_eq!(
            config.token_for_package("@doki-land/nifty"),
            Some("npm_test_token".to_string())
        );
    }

    #[test]
    fn resolves_scoped_registry_and_auth() {
        let config = NpmRc::from_text(
            "@my:registry=https://npm.pkg.github.com/\n//npm.pkg.github.com/:_authToken=ghp_test\n",
        );
        assert_eq!(
            config.registry_for_package("@my/pkg"),
            "https://npm.pkg.github.com"
        );
        assert_eq!(config.token_for_package("@my/pkg"), Some("ghp_test".to_string()));
    }

    #[test]
    fn project_npmrc_overrides_user_default_registry() {
        let mut merged = NpmRc::from_text(
            "registry=https://registry.npmjs.org/\n//registry.npmjs.org/:_authToken=npm_user\n",
        );
        merged.merge(NpmRc::from_text("registry=https://registry.example.com/\n"));
        assert_eq!(merged.default_registry, "https://registry.example.com");
        assert_eq!(merged.default_token(), None);
    }
}
