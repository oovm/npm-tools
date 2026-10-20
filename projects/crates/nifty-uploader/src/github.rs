use std::time::Duration;

use request::{HttpTransport, HttpTransportConfig, Method, Request, Response, Transport, Url};
use serde_json::Value;

use crate::Result;

const USER_AGENT: &str = "nifty-uploader";
const ACCEPT: &str = "application/vnd.github+json";
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(300);

pub struct GitHubClient {
    transport: HttpTransport,
    token: String,
}

impl GitHubClient {
    pub fn new(token: &str) -> Result<Self> {
        let token = token.trim();
        if token.is_empty() {
            return Err("GitHub token must not be empty".to_string());
        }
        let config = HttpTransportConfig { timeout: UPLOAD_TIMEOUT, ..HttpTransportConfig::default() };
        Ok(Self { transport: HttpTransport::with_config(config), token: token.to_string() })
    }

    pub fn get_json(&self, url: &str) -> Result<Value> {
        let request = self.request(Method::Get, url)?;
        let response = self.transport.send(&request).map_err(|err| format!("GitHub GET failed: {err}"))?;
        parse_json_response(response)
    }

    pub fn post_json(&self, url: &str, body: &Value) -> Result<Value> {
        let body = serde_json::to_vec(body).map_err(|err| format!("serialize GitHub JSON body: {err}"))?;
        let request = self.request(Method::Post, url)?.with_header("Content-Type", "application/json").with_body(body);
        let response = self.transport.send(&request).map_err(|err| format!("GitHub POST failed: {err}"))?;
        parse_json_response(response)
    }

    pub fn upload_bytes(&self, url: &str, content_type: &str, bytes: &[u8]) -> Result<Value> {
        let request = self.request(Method::Post, url)?.with_header("Content-Type", content_type).with_body(bytes.to_vec());
        let response = self.transport.send(&request).map_err(|err| format!("GitHub upload failed: {err}"))?;
        parse_json_response(response)
    }

    fn request(&self, method: Method, url: &str) -> Result<Request> {
        let url = Url::parse(url).map_err(|err| format!("invalid GitHub URL `{url}`: {err}"))?;
        Ok(Request::new(method, url)
            .with_header("Accept", ACCEPT)
            .with_header("User-Agent", USER_AGENT)
            .with_header("Authorization", format!("Bearer {}", self.token)))
    }
}

fn parse_json_response(response: Response) -> Result<Value> {
    let status = response.status;
    if status == 404 {
        return Err("GitHub resource not found".to_string());
    }
    if !response.is_success() {
        return Err(format!("GitHub API error: {status} {}", String::from_utf8_lossy(&response.body)));
    }
    if response.body.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_slice(&response.body).map_err(|err| format!("parse GitHub JSON: {err}"))
}

pub fn release_asset_already_exists(message: &str) -> bool {
    message.contains("already_exists")
}

pub fn split_repo(repo: &str) -> Result<(String, String)> {
    let repo = repo.trim().trim_end_matches(".git");
    let (owner, name) = repo.split_once('/').ok_or_else(|| format!("invalid repo `{repo}`, expected owner/name"))?;
    Ok((owner.to_string(), name.to_string()))
}

pub fn api_release_by_tag(client: &GitHubClient, owner: &str, repo: &str, tag: &str) -> Result<Option<Value>> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");
    match client.get_json(&url) {
        Ok(value) => Ok(Some(value)),
        Err(err) if err.contains("not found") => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn api_create_release(
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    tag: &str,
    name: &str,
    body: &str,
    draft: bool,
) -> Result<Value> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases");
    let payload = serde_json::json!({
        "tag_name": tag,
        "name": name,
        "body": body,
        "draft": draft,
        "generate_release_notes": false,
    });
    client.post_json(&url, &payload)
}

pub fn api_upload_asset(
    client: &GitHubClient,
    owner: &str,
    repo: &str,
    release_id: u64,
    asset_name: &str,
    bytes: &[u8],
) -> Result<Value> {
    let url = format!(
        "https://uploads.github.com/repos/{owner}/{repo}/releases/{release_id}/assets?name={}",
        urlencoding_encode(asset_name)
    );
    client.upload_bytes(&url, "application/octet-stream", bytes)
}

fn urlencoding_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

pub fn release_id(value: &Value) -> Result<u64> {
    value.get("id").and_then(Value::as_u64).ok_or_else(|| "GitHub release response missing id".to_string())
}

#[cfg(test)]
mod tests {
    use super::split_repo;

    #[test]
    fn splits_owner_repo() {
        assert_eq!(split_repo("oovm/npm-tools").unwrap(), ("oovm".to_string(), "npm-tools".to_string()));
    }
}
