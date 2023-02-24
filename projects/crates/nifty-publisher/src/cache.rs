use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::trust::{TRUST_ENV, TRUST_FILE, TRUST_REPO};
use crate::Result;

pub const CACHE_DIR_NAME: &str = ".cache";
pub const CACHE_FILE_NAME: &str = "npm-placeholder.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustExpect {
    pub repo: String,
    pub file: String,
    pub env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TrustCacheEntry {
    pub listed_at: String,
    pub configs: Vec<Value>,
    pub matches: bool,
    pub match_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PackageCacheEntry {
    pub version: Option<String>,
    pub version_at: Option<String>,
    pub trust: Option<TrustCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlaceholderCache {
    pub version: u32,
    pub trust_expect: TrustExpect,
    pub packages: BTreeMap<String, PackageCacheEntry>,
}

impl Default for PlaceholderCache {
    fn default() -> Self {
        Self {
            version: 1,
            trust_expect: TrustExpect {
                repo: TRUST_REPO.to_string(),
                file: TRUST_FILE.to_string(),
                env: TRUST_ENV.to_string(),
            },
            packages: BTreeMap::new(),
        }
    }
}

impl PlaceholderCache {
    pub fn path(workspace_root: &Path) -> PathBuf {
        workspace_root.join(CACHE_DIR_NAME).join(CACHE_FILE_NAME)
    }

    pub fn load(workspace_root: &Path) -> Self {
        let path = Self::path(workspace_root);
        if let Ok(cache) = read_cache_file(&path) {
            return cache;
        }
        let legacy = workspace_root
            .join(CACHE_DIR_NAME)
            .join("placeholder-npm-cache.json");
        if legacy != path {
            if let Ok(cache) = read_cache_file(&legacy) {
                let _ = save_cache(workspace_root, &cache);
                return cache;
            }
        }
        Self::default()
    }

    pub fn save(&self, workspace_root: &Path) -> Result<()> {
        save_cache(workspace_root, self)
    }

    pub fn package_mut(&mut self, name: &str) -> &mut PackageCacheEntry {
        self.packages.entry(name.to_string()).or_default()
    }

    pub fn trust_matches_cached(&self, name: &str, refresh: bool) -> bool {
        if refresh {
            return false;
        }
        let entry = match self.packages.get(name).and_then(|pkg| pkg.trust.as_ref()) {
            Some(entry) => entry,
            None => return false,
        };
        if !entry.matches {
            return false;
        }
        classify_configs(&entry.configs).matches
    }

    pub fn record_trust_list(&mut self, name: &str, configs: Vec<Value>) {
        let classification = classify_configs(&configs);
        let trust = TrustCacheEntry {
            listed_at: now_iso(),
            configs,
            matches: classification.matches,
            match_kind: classification.match_kind,
        };
        self.package_mut(name).trust = Some(trust);
    }

    pub fn record_version(&mut self, name: &str, version: &str) {
        let entry = self.package_mut(name);
        entry.version = Some(version.to_string());
        entry.version_at = Some(now_iso());
    }

    pub fn cached_version(&self, name: &str) -> Option<&str> {
        self.packages
            .get(name)
            .and_then(|entry| entry.version.as_deref())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSource {
    Live,
    Cache,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVersion {
    pub version: Option<String>,
    pub source: VersionSource,
    pub live_miss: bool,
}

pub fn resolve_published_version(
    cache: &mut PlaceholderCache,
    name: &str,
    live_version: Option<String>,
) -> ResolvedVersion {
    if let Some(version) = live_version {
        cache.record_version(name, &version);
        return ResolvedVersion {
            version: Some(version),
            source: VersionSource::Live,
            live_miss: false,
        };
    }
    if let Some(version) = cache.cached_version(name).map(str::to_string) {
        return ResolvedVersion {
            version: Some(version),
            source: VersionSource::Cache,
            live_miss: true,
        };
    }
    ResolvedVersion {
        version: None,
        source: VersionSource::None,
        live_miss: true,
    }
}

pub fn should_skip_publish(
    cache: &PlaceholderCache,
    name: &str,
    target_version: &str,
    resolved: &ResolvedVersion,
    refresh: bool,
    dry_run: bool,
) -> bool {
    if dry_run || refresh {
        return false;
    }
    if resolved.version.as_deref() == Some(target_version) {
        return true;
    }
    cache.cached_version(name) == Some(target_version)
}

fn read_cache_file(path: &Path) -> Result<PlaceholderCache> {
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let data: PlaceholderCache = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    if data.trust_expect.repo != TRUST_REPO
        || data.trust_expect.file != TRUST_FILE
        || data.trust_expect.env != TRUST_ENV
    {
        return Ok(PlaceholderCache::default());
    }
    Ok(data)
}

fn save_cache(workspace_root: &Path, cache: &PlaceholderCache) -> Result<()> {
    let path = PlaceholderCache::path(workspace_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let mut next = cache.clone();
    next.version = 1;
    next.trust_expect = TrustExpect {
        repo: TRUST_REPO.to_string(),
        file: TRUST_FILE.to_string(),
        env: TRUST_ENV.to_string(),
    };
    let text = serde_json::to_string_pretty(&next).map_err(|err| err.to_string())?;
    fs::write(path, format!("{text}\n")).map_err(|err| err.to_string())?;
    Ok(())
}

pub struct TrustClassification {
    pub matches: bool,
    pub match_kind: String,
}

pub fn classify_configs(configs: &[Value]) -> TrustClassification {
    if configs.iter().any(trust_exact) {
        return TrustClassification {
            matches: true,
            match_kind: "exact".to_string(),
        };
    }
    if configs.iter().any(trust_matches) {
        return TrustClassification {
            matches: true,
            match_kind: "loose".to_string(),
        };
    }
    if configs.is_empty() {
        return TrustClassification {
            matches: false,
            match_kind: "none".to_string(),
        };
    }
    TrustClassification {
        matches: false,
        match_kind: "mismatch".to_string(),
    }
}

fn trust_exact(config: &Value) -> bool {
    trust_matches(config) && trust_fields(config).env == TRUST_ENV
}

fn trust_matches(config: &Value) -> bool {
    if let Some(raw) = config.get("raw").and_then(|value| value.as_str()) {
        return raw.contains(TRUST_REPO) && raw.contains(TRUST_FILE) && raw.contains(TRUST_ENV);
    }
    let fields = trust_fields(config);
    fields.repo == TRUST_REPO && fields.file == TRUST_FILE
}

struct TrustFields {
    repo: String,
    file: String,
    env: String,
}

fn trust_fields(config: &Value) -> TrustFields {
    let claims = config.get("claims").unwrap_or(config);
    TrustFields {
        repo: pick_string(config, claims, &["repository", "repo"]),
        file: pick_string(config, claims, &["file", "workflow", "workflowFile"]),
        env: pick_string(config, claims, &["environment", "env"]),
    }
}

fn pick_string(config: &Value, claims: &Value, keys: &[&str]) -> String {
    for &key in keys {
        if let Some(value) = config.get(key).and_then(|value| value.as_str()) {
            return value.to_string();
        }
        if let Some(value) = claims.get(key).and_then(|value| value.as_str()) {
            return value.to_string();
        }
        if let Some(workflow) = claims.get("workflow_ref").and_then(Value::as_object) {
            if let Some(value) = workflow.get(key).and_then(|value| value.as_str()) {
                return value.to_string();
            }
        }
    }
    String::new()
}

fn now_iso() -> String {
    // ISO-like timestamp without chrono dependency.
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{PlaceholderCache, resolve_published_version, should_skip_publish, VersionSource};
    use tempfile::TempDir;

    #[test]
    fn saves_and_loads_cache_file() {
        let dir = TempDir::new().expect("tempdir");
        let mut cache = PlaceholderCache::default();
        cache.record_version("@doki-land/nifty", "0.0.1");
        cache.save(dir.path()).expect("save");
        let loaded = PlaceholderCache::load(dir.path());
        assert_eq!(loaded.cached_version("@doki-land/nifty"), Some("0.0.1"));
    }

    #[test]
    fn skip_publish_when_versions_match() {
        let mut cache = PlaceholderCache::default();
        cache.record_version("@doki-land/nifty", "0.0.1");
        let resolved = resolve_published_version(&mut cache, "@doki-land/nifty", Some("0.0.1".into()));
        assert!(should_skip_publish(&cache, "@doki-land/nifty", "0.0.1", &resolved, false, false));
        assert_eq!(resolved.source, VersionSource::Live);
    }
}
