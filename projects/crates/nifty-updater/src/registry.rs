use serde_json::Value;

use crate::Result;

const CRATES_IO_USER_AGENT: &str = "nifty-updater (https://github.com/oovm/npm-tools)";
const NPM_REGISTRY: &str = "https://registry.npmjs.org";

/// Latest semver string for a crates.io package.
pub fn fetch_crate_latest(name: &str) -> Result<String> {
    let url = format!("https://crates.io/api/v1/crates/{}", urlencoding::encode(name));
    let value = fetch_json(&url)?;
    let version = value
        .pointer("/crate/newest_version")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("crates.io: missing newest_version for `{name}`"))?;
    Ok(version.to_string())
}

/// Latest semver string for an npm package (`registry.npmjs.org`).
pub fn fetch_npm_latest(name: &str) -> Result<String> {
    let url = format!("{NPM_REGISTRY}/{}/latest", urlencoding::encode(name));
    let value = fetch_json(&url)?;
    let version = value.get("version").and_then(Value::as_str).ok_or_else(|| format!("npm: missing version for `{name}`"))?;
    Ok(version.to_string())
}

fn fetch_json(url: &str) -> Result<Value> {
    let response = ureq::get(url)
        .set("Accept", "application/json")
        .set("User-Agent", CRATES_IO_USER_AGENT)
        .call()
        .map_err(|err| format!("registry request failed ({url}): {err}"))?;
    let status = response.status();
    let body = response.into_string().map_err(|err| err.to_string())?;
    if !(200..300).contains(&status) {
        return Err(format!("registry {status} ({url}): {body}"));
    }
    serde_json::from_str(&body).map_err(|err| format!("registry JSON parse ({url}): {err}"))
}
