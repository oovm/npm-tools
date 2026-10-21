use boost_http::{HttpTransport, Method, Request, Transport, Url};
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
    let parsed = Url::parse(url).map_err(|err| format!("invalid registry URL `{url}`: {err}"))?;
    let request = Request::new(Method::Get, parsed)
        .with_header("Accept", "application/json")
        .with_header("User-Agent", CRATES_IO_USER_AGENT);
    let response = HttpTransport::new().send(&request).map_err(|err| format!("registry request failed ({url}): {err}"))?;
    if !response.is_success() {
        return Err(format!("registry {} ({url}): {}", response.status, String::from_utf8_lossy(&response.body)));
    }
    serde_json::from_slice(&response.body).map_err(|err| format!("registry JSON parse ({url}): {err}"))
}
