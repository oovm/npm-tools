//! 组合本地 author 映射与 GitHub API 查询。

use nifty_types::{
    GithubAuthor, github_from_noreply_email, load_author_map_from_json, merge_authors, resolve_github_author,
};

use crate::client::{Result, search_user_by_email, user_by_login};

/// 按 email 解析 GitHub 用户：noreply → 本地映射 → 可选 API 搜索 / 补全。
pub fn lookup_user_by_email(
    email: &str,
    author_map_json: &str,
    token: Option<&str>,
    fetch: bool,
) -> Result<Option<GithubAuthor>> {
    if let Some(from_noreply) = github_from_noreply_email(email) {
        return Ok(Some(enrich_author(from_noreply, token, fetch)?));
    }
    let map = load_author_map_from_json(author_map_json);
    if let Some(mapped) = resolve_github_author(email, &map) {
        return Ok(Some(enrich_author(mapped, token, fetch)?));
    }
    if let Some(token) = token {
        if let Some(found) = search_user_by_email(email, token)? {
            return Ok(Some(enrich_author(found, Some(token), fetch)?));
        }
    }
    Ok(None)
}

fn enrich_author(author: GithubAuthor, token: Option<&str>, fetch: bool) -> Result<GithubAuthor> {
    if !fetch && author.id.is_some() && author.login.is_some() {
        return Ok(author);
    }
    if !fetch && author.id.is_some() {
        return Ok(author);
    }
    if let Some(login) = &author.login {
        if author.id.is_none() || fetch {
            let fetched = user_by_login(login, token)?;
            return Ok(merge_authors(&author, &fetched));
        }
    }
    Ok(author)
}
