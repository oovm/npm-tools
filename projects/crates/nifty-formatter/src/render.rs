//! Markdown bullets and author @mentions for release notes.

use std::collections::HashMap;

use nifty_types::{GithubAuthor, Section, display_login, profile_url, resolve_github_author, section_meta};

/// Markdown author mention.
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

/// Single commit bullet: `- subject (@user)`.
pub fn commit_bullet(body: &str, email: &str, author_name: &str, map: &HashMap<String, GithubAuthor>) -> String {
    format!("- {} ({})", body, author_mention(email, author_name, map))
}

/// Render commit bullets for one gitmoji section.
pub fn render_section_bullets(
    section: Section,
    commits: &[(String, String, String)],
    map: &HashMap<String, GithubAuthor>,
) -> String {
    let (_, empty) = section_meta(section);
    if commits.is_empty() {
        return empty.to_string();
    }
    commits.iter().map(|(body, email, author)| commit_bullet(body, email, author, map)).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::commit_bullet;

    #[test]
    fn commit_bullet_uses_email_local_part() {
        let bullet = commit_bullet("Fix bug", "dev@example.com", "Dev", &HashMap::new());
        assert_eq!(bullet, "- Fix bug (@Dev)");
    }
}
