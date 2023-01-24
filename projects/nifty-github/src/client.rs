//! GitHub REST API client (`api.github.com`).

use nifty_core::GithubAuthor;
use serde_json::Value;

pub type Result<T> = std::result::Result<T, String>;

const USER_AGENT: &str = "nifty-github";
const ACCEPT: &str = "application/vnd.github+json";

/// 按 login 查询用户（公开接口，token 可选，用于提高限额）。
pub fn user_by_login(login: &str, token: Option<&str>) -> Result<GithubAuthor> {
    let login = login.trim();
    if login.is_empty() {
        return Err("login must not be empty".to_string());
    }
    let url = format!("https://api.github.com/users/{login}");
    let body = get_json(&url, token)?;
    parse_user_json(&body, Some(login))
}

/// 按 email 搜索用户（需要 token，受 GitHub 隐私策略限制）。
pub fn search_user_by_email(email: &str, token: &str) -> Result<Option<GithubAuthor>> {
    let email = email.trim();
    if email.is_empty() {
        return Err("email must not be empty".to_string());
    }
    let token = token.trim();
    if token.is_empty() {
        return Err("GitHub token is required for email search".to_string());
    }
    let query = format!("{} in:email", email);
    let url = format!("https://api.github.com/search/users?q={}", urlencoding::encode(&query));
    let body = get_json(&url, Some(token))?;
    let items = body.get("items").and_then(Value::as_array);
    let Some(items) = items else {
        return Ok(None);
    };
    let first = items.first().and_then(Value::as_object);
    let Some(first) = first else {
        return Ok(None);
    };
    let id = first.get("id").and_then(Value::as_u64);
    let login = first.get("login").and_then(Value::as_str).map(str::to_string);
    if id.is_none() && login.is_none() {
        return Ok(None);
    }
    Ok(Some(GithubAuthor { id, login }))
}

pub fn parse_user_json(body: &Value, fallback_login: Option<&str>) -> Result<GithubAuthor> {
    let id = body.get("id").and_then(Value::as_u64);
    let login = body
        .get("login")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| fallback_login.map(str::to_string));
    if id.is_none() && login.is_none() {
        return Err("GitHub user response missing id and login".to_string());
    }
    Ok(GithubAuthor { id, login })
}

fn get_json(url: &str, token: Option<&str>) -> Result<Value> {
    let mut request = ureq::get(url).set("Accept", ACCEPT).set("User-Agent", USER_AGENT);
    if let Some(token) = token {
        let token = token.trim();
        if !token.is_empty() {
            request = request.set("Authorization", &format!("Bearer {token}"));
        }
    }
    let response = request.call().map_err(|err| format!("GitHub API request failed: {err}"))?;
    let status = response.status();
    if status == 404 {
        return Err("GitHub resource not found".to_string());
    }
    if !(200..300).contains(&status) {
        return Err(format!("GitHub API error: {} {}", status, response.status_text()));
    }
    response
        .into_json()
        .map_err(|err| format!("parse GitHub API JSON: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_user_response() {
        let body = json!({ "id": 1, "login": "octocat" });
        let user = parse_user_json(&body, None).unwrap();
        assert_eq!(user.id, Some(1));
        assert_eq!(user.login.as_deref(), Some("octocat"));
    }
}
