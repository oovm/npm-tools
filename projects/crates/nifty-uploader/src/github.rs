use serde_json::Value;
use ureq::Agent;

use crate::Result;

const USER_AGENT: &str = "nifty-uploader";
const ACCEPT: &str = "application/vnd.github+json";

pub struct GitHubClient {
    agent: Agent,
    token: String,
}

impl GitHubClient {
    pub fn new(token: &str) -> Result<Self> {
        let token = token.trim();
        if token.is_empty() {
            return Err("GitHub token must not be empty".to_string());
        }
        Ok(Self {
            agent: Agent::new(),
            token: token.to_string(),
        })
    }

    pub fn get_json(&self, url: &str) -> Result<Value> {
        let response = self
            .agent
            .get(url)
            .set("Accept", ACCEPT)
            .set("User-Agent", USER_AGENT)
            .set("Authorization", &format!("Bearer {}", self.token))
            .call()
            .map_err(|err| format!("GitHub GET failed: {err}"))?;
        parse_json_response(response)
    }

    pub fn post_json(&self, url: &str, body: &Value) -> Result<Value> {
        let response = self
            .agent
            .post(url)
            .set("Accept", ACCEPT)
            .set("User-Agent", USER_AGENT)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|err| format!("GitHub POST failed: {err}"))?;
        parse_json_response(response)
    }

    pub fn upload_bytes(&self, url: &str, content_type: &str, bytes: &[u8]) -> Result<Value> {
        let response = self
            .agent
            .post(url)
            .set("Accept", ACCEPT)
            .set("User-Agent", USER_AGENT)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Content-Type", content_type)
            .send(bytes)
            .map_err(|err| format!("GitHub upload failed: {err}"))?;
        parse_json_response(response)
    }
}

fn parse_json_response(response: ureq::Response) -> Result<Value> {
    let status = response.status();
    let status_text = response.status_text().to_string();
    let body = response.into_string().unwrap_or_default();
    if status == 404 {
        return Err("GitHub resource not found".to_string());
    }
    if !(200..300).contains(&status) {
        return Err(format!("GitHub API error: {} {} {}", status, status_text, body));
    }
    if body.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&body).map_err(|err| format!("parse GitHub JSON: {err}"))
}

pub fn release_asset_already_exists(message: &str) -> bool {
    message.contains("already_exists")
}

pub fn split_repo(repo: &str) -> Result<(String, String)> {
    let repo = repo.trim().trim_end_matches(".git");
    let (owner, name) = repo
        .split_once('/')
        .ok_or_else(|| format!("invalid repo `{repo}`, expected owner/name"))?;
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
    value
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| "GitHub release response missing id".to_string())
}

#[cfg(test)]
mod tests {
    use super::split_repo;

    #[test]
    fn splits_owner_repo() {
        assert_eq!(split_repo("oovm/npm-tools").unwrap(), ("oovm".to_string(), "npm-tools".to_string()));
    }
}
