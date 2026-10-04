use semver::{Version, VersionReq};

use crate::Result;

/// True when `latest` is strictly newer than `current` (both must parse as semver).
pub fn is_upgrade_available(current: &str, latest: &str) -> bool {
    match (Version::parse(current), Version::parse(latest)) {
        (Ok(current), Ok(latest)) => latest > current,
        _ => latest != current,
    }
}

/// Strip npm range operators (`^`, `~`, `>=`, …) for semver comparison.
pub fn npm_spec_version(spec: &str) -> &str {
    let trimmed = spec.trim();
    let trimmed = trimmed
        .strip_prefix('^')
        .or_else(|| trimmed.strip_prefix('~'))
        .or_else(|| trimmed.strip_prefix('='))
        .unwrap_or(trimmed);
    trimmed
        .strip_prefix(">=")
        .or_else(|| trimmed.strip_prefix("<="))
        .or_else(|| trimmed.strip_prefix('>'))
        .or_else(|| trimmed.strip_prefix('<'))
        .unwrap_or(trimmed)
        .trim()
}

/// True when `latest` is not already allowed by an npm range spec.
pub fn npm_upgrade_needed(spec: &str, latest: &str) -> Result<bool> {
    if VersionReq::parse(spec).is_ok() {
        let latest_ver = Version::parse(latest).map_err(|err| err.to_string())?;
        return Ok(!VersionReq::parse(spec).map_err(|err| err.to_string())?.matches(&latest_ver));
    }
    let current = npm_spec_version(spec);
    Ok(is_upgrade_available(current, latest))
}

/// Cargo manifest version requirement after bump (caret on major when zero-major).
pub fn cargo_version_req(latest: &str) -> String {
    match Version::parse(latest) {
        Ok(version) if version.major == 0 => format!("={latest}"),
        Ok(version) => format!("^{}.{}", version.major, version.minor),
        Err(_) => latest.to_string(),
    }
}

/// npm package.json spec after bump.
pub fn npm_version_spec(latest: &str) -> String {
    format!("^{latest}")
}
