//! GitHub 作者解析（noreply 邮箱与 `author-github.json` 映射）。

use std::collections::HashMap;

use serde_json::Value;

/// GitHub 用户标识（numeric id 优先用于稳定头像 URL）。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GithubAuthor {
    /// GitHub numeric user id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    /// GitHub login handle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
}

/// 从 `author-github.json` 文本加载 email → GitHub 映射（email 键小写）。
pub fn load_author_map_from_json(text: &str) -> HashMap<String, GithubAuthor> {
    let text = text.trim_start_matches('\u{feff}');
    let raw: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(_) => return HashMap::new(),
    };
    let Some(object) = raw.as_object() else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for (email, entry) in object {
        if let Some(parsed) = parse_author_entry(entry) {
            out.insert(email.trim().to_lowercase(), parsed);
        }
    }
    out
}

/// 解析映射文件中的单条 entry。
pub fn parse_author_entry(value: &Value) -> Option<GithubAuthor> {
    match value {
        Value::String(login) => {
            let login = login.trim();
            if login.is_empty() {
                None
            } else {
                Some(GithubAuthor { id: None, login: Some(login.to_string()) })
            }
        }
        Value::Number(number) => number.as_u64().map(|id| GithubAuthor { id: Some(id), login: None }),
        Value::Object(object) => {
            let login = object.get("login").and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty());
            let id = object.get("id").and_then(Value::as_u64);
            if login.is_some() || id.is_some() {
                Some(GithubAuthor { id, login: login.map(str::to_string) })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 解析 GitHub noreply 邮箱。
pub fn github_from_noreply_email(email: &str) -> Option<GithubAuthor> {
    if let Some((id, login)) = split_noreply_with_id(email.trim()) {
        return Some(GithubAuthor { id: Some(id), login: Some(login) });
    }
    parse_noreply_login_only(email.trim())
}

fn split_noreply_with_id(email: &str) -> Option<(u64, String)> {
    let lower = email.to_ascii_lowercase();
    let suffix = "@users.noreply.github.com";
    if !lower.ends_with(suffix) {
        return None;
    }
    let local = email.get(..email.len() - suffix.len())?;
    let (id_part, login) = local.split_once('+')?;
    let id = id_part.parse().ok()?;
    if login.is_empty() {
        return None;
    }
    Some((id, login.to_string()))
}

fn parse_noreply_login_only(email: &str) -> Option<GithubAuthor> {
    let lower = email.to_ascii_lowercase();
    let suffix = "@users.noreply.github.com";
    if !lower.ends_with(suffix) || email.contains('+') {
        return None;
    }
    let login = email.get(..email.len() - suffix.len())?.trim();
    if login.is_empty() {
        return None;
    }
    Some(GithubAuthor { id: None, login: Some(login.to_string()) })
}

/// 合并两位贡献者信息（补全缺失的 id / login）。
pub fn merge_authors(existing: &GithubAuthor, incoming: &GithubAuthor) -> GithubAuthor {
    GithubAuthor {
        id: existing.id.or(incoming.id),
        login: existing.login.clone().or_else(|| incoming.login.clone()),
    }
}

/// 合并 noreply 解析与本地映射。
pub fn resolve_github_author(email: &str, map: &HashMap<String, GithubAuthor>) -> Option<GithubAuthor> {
    if let Some(from_noreply) = github_from_noreply_email(email) {
        return Some(from_noreply);
    }
    map.get(&email.trim().to_lowercase()).cloned()
}

/// 展示用 login。
pub fn display_login(author: &GithubAuthor) -> String {
    if let Some(login) = &author.login {
        return login.clone();
    }
    if let Some(id) = author.id {
        return format!("user-{id}");
    }
    "unknown".to_string()
}

/// GitHub profile URL。
pub fn profile_url(author: &GithubAuthor) -> String {
    if let Some(login) = &author.login {
        return format!("https://github.com/{login}");
    }
    if let Some(id) = author.id {
        return format!("https://github.com/user/{id}");
    }
    "#".to_string()
}

/// 头像 URL（优先 numeric id）。
pub fn avatar_url(author: &GithubAuthor) -> String {
    if let Some(id) = author.id {
        return format!("https://avatars.githubusercontent.com/u/{id}?s=100");
    }
    if let Some(login) = &author.login {
        return format!("https://github.com/{login}.png?s=100");
    }
    String::new()
}
