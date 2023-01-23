//! release 参考稿渲染（gitmoji 分组 + 贡献者 @mention）。

use std::collections::HashMap;

use crate::author::{GithubAuthor, display_login, profile_url, resolve_github_author};
use crate::gitmoji::{Section, section_meta};

/// Markdown 作者提及。
pub fn author_mention(email: &str, author_name: &str, map: &HashMap<String, GithubAuthor>) -> String {
    if let Some(github) = resolve_github_author(email, map) {
        let label = format!("@{}", display_login(&github));
        return format!("[{label}]({})", profile_url(&github));
    }
    let name = author_name.trim();
    if !name.is_empty() {
        return format!("@{}", name.replace(' ', ""));
    }
    if let Some(local) = email.split('@').next().map(str::trim).filter(|part| !part.is_empty()) {
        return format!("@{local}");
    }
    "@unknown".to_string()
}

/// 单条 commit bullet：`- subject (@user)`。
pub fn commit_bullet(body: &str, email: &str, author_name: &str, map: &HashMap<String, GithubAuthor>) -> String {
    format!("- {} ({})", body, author_mention(email, author_name, map))
}

/// 将 commit 列表按 gitmoji section 渲染为 reference markdown 片段。
pub fn render_section_bullets(
    section: Section,
    commits: &[(String, String, String)],
    map: &HashMap<String, GithubAuthor>,
) -> String {
    let (_, empty) = section_meta(section);
    if commits.is_empty() {
        return empty.to_string();
    }
    commits
        .iter()
        .map(|(body, email, author)| commit_bullet(body, email, author, map))
        .collect::<Vec<_>>()
        .join("\n")
}
