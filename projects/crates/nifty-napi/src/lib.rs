//! Node-API export surface for Nifty (gitmoji + gix git helpers).

use std::collections::HashMap;
use std::path::Path;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use nifty_formatter::{author_mention, commit_bullet, format_subject as fmt_subject};
use nifty_git::{
    CommitRecord as CoreCommit, RangeInfo as CoreRange, TagInfo as CoreTag, collect_commits, detect_github_repo,
    discover_root, format_tag_list, list_tag_infos, list_version_tags, parse_github_remote_repo, resolve_range,
};
use nifty_types::{
    GithubAuthor as CoreAuthor, avatar_url, display_login, github_from_noreply_email, known_gitmojis,
    leading_gitmoji, load_author_map_from_json, parse_subject, profile_url, resolve_github_author,
    section_for_gitmoji, section_name, strip_gitmoji, validate_subject,
};

#[napi(object)]
pub struct GithubAuthor {
    #[napi(ts_type = "bigint")]
    pub id: Option<i64>,
    pub login: Option<String>,
}

#[napi(object)]
pub struct ParsedSubject {
    pub gitmoji: Option<String>,
    pub body: String,
    pub section: String,
}

#[napi(object)]
pub struct TagInfo {
    pub name: String,
    pub short_hash: String,
}

#[napi(object)]
pub struct RangeInfo {
    pub version: String,
    pub from_ref: Option<String>,
    pub to_ref: String,
}

#[napi(object)]
pub struct CommitRecord {
    pub hash: String,
    pub email: String,
    pub author: String,
    pub subject: String,
    pub body: String,
    pub gitmoji: Option<String>,
    pub section: String,
}

fn to_napi_author(author: CoreAuthor) -> GithubAuthor {
    GithubAuthor {
        id: author.id.map(|id| id as i64),
        login: author.login,
    }
}

fn from_napi_author(author: &GithubAuthor) -> CoreAuthor {
    CoreAuthor {
        id: author.id.map(|id| id as u64),
        login: author.login.clone(),
    }
}

fn load_map(json: &str) -> HashMap<String, CoreAuthor> {
    load_author_map_from_json(json)
}

fn map_err<T>(result: std::result::Result<T, String>) -> Result<T> {
    result.map_err(|message| Error::from_reason(message))
}

fn to_napi_tag(tag: CoreTag) -> TagInfo {
    TagInfo {
        name: tag.name,
        short_hash: tag.short_hash,
    }
}

fn to_napi_range(range: CoreRange) -> RangeInfo {
    RangeInfo {
        version: range.version,
        from_ref: range.from_ref,
        to_ref: range.to_ref,
    }
}

fn to_napi_commit(commit: CoreCommit) -> CommitRecord {
    CommitRecord {
        hash: commit.hash,
        email: commit.email,
        author: commit.author,
        subject: commit.subject,
        body: commit.body,
        gitmoji: commit.gitmoji,
        section: commit.section,
    }
}

// --- gitmoji ---

#[napi]
pub fn gitmoji_known_gitmojis() -> Vec<String> {
    known_gitmojis().iter().map(|emoji| emoji.to_string()).collect()
}

#[napi]
pub fn gitmoji_validate_subject(subject: String) -> bool {
    validate_subject(&subject)
}

#[napi]
pub fn gitmoji_parse_subject(subject: String) -> ParsedSubject {
    let parsed = parse_subject(&subject);
    ParsedSubject {
        gitmoji: parsed.gitmoji,
        body: parsed.body,
        section: section_name(parsed.section).to_string(),
    }
}

#[napi]
pub fn gitmoji_format_subject(gitmoji: String, body: String) -> String {
    fmt_subject(&gitmoji, &body)
}

#[napi]
pub fn gitmoji_strip_gitmoji(subject: String) -> String {
    strip_gitmoji(&subject)
}

#[napi]
pub fn gitmoji_leading_gitmoji(subject: String) -> Option<String> {
    leading_gitmoji(&subject).map(str::to_string)
}

#[napi]
pub fn gitmoji_section_for_gitmoji(gitmoji: Option<String>) -> String {
    section_name(section_for_gitmoji(gitmoji.as_deref())).to_string()
}

#[napi]
pub fn gitmoji_parse_noreply_email(email: String) -> Option<GithubAuthor> {
    github_from_noreply_email(&email).map(to_napi_author)
}

#[napi]
pub fn gitmoji_resolve_github_author(email: String, author_map_json: String) -> Option<GithubAuthor> {
    let map = load_map(&author_map_json);
    resolve_github_author(&email, &map).map(to_napi_author)
}

#[napi]
pub fn gitmoji_display_login(author: GithubAuthor) -> String {
    display_login(&from_napi_author(&author))
}

#[napi]
pub fn gitmoji_profile_url(author: GithubAuthor) -> String {
    profile_url(&from_napi_author(&author))
}

#[napi]
pub fn gitmoji_avatar_url(author: GithubAuthor) -> String {
    avatar_url(&from_napi_author(&author))
}

#[napi]
pub fn gitmoji_author_mention(email: String, author_name: String, author_map_json: String) -> String {
    let map = load_map(&author_map_json);
    author_mention(&email, &author_name, &map)
}

#[napi]
pub fn gitmoji_commit_bullet(body: String, email: String, author_name: String, author_map_json: String) -> String {
    let map = load_map(&author_map_json);
    commit_bullet(&body, &email, &author_name, &map)
}

// --- git ---

#[napi]
pub fn git_discover_root(start_path: String) -> Result<String> {
    map_err(discover_root(Path::new(&start_path)).map(|path| path.display().to_string()))
}

#[napi]
pub fn git_list_version_tags(repo_root: String) -> Result<Vec<String>> {
    map_err(list_version_tags(Path::new(&repo_root)))
}

#[napi]
pub fn git_list_tag_infos(repo_root: String) -> Result<Vec<TagInfo>> {
    map_err(list_tag_infos(Path::new(&repo_root)).map(|tags| tags.into_iter().map(to_napi_tag).collect()))
}

#[napi]
pub fn git_format_tag_list(repo_root: String) -> Result<String> {
    map_err(format_tag_list(Path::new(&repo_root)))
}

#[napi]
pub fn git_resolve_range(
    repo_root: String,
    version: Option<String>,
    from_ref: Option<String>,
    to_ref: Option<String>,
) -> Result<RangeInfo> {
    map_err(
        resolve_range(
            Path::new(&repo_root),
            version.as_deref(),
            from_ref.as_deref(),
            to_ref.as_deref(),
        )
        .map(to_napi_range),
    )
}

#[napi]
pub fn git_collect_commits(repo_root: String, from_ref: Option<String>, to_ref: String) -> Result<Vec<CommitRecord>> {
    map_err(
        collect_commits(Path::new(&repo_root), from_ref.as_deref(), &to_ref)
            .map(|commits| commits.into_iter().map(to_napi_commit).collect()),
    )
}

#[napi]
pub fn git_detect_github_repo(repo_root: String) -> Result<Option<String>> {
    map_err(detect_github_repo(Path::new(&repo_root)))
}

#[napi]
pub fn git_parse_github_remote(url: String) -> Option<String> {
    parse_github_remote_repo(&url)
}

// --- github ---

#[napi]
pub fn github_user_by_login(login: String, token: Option<String>) -> Result<GithubAuthor> {
    map_err(nifty_github::user_by_login(&login, token.as_deref()).map(to_napi_author))
}

#[napi]
pub fn github_search_user_by_email(email: String, token: String) -> Result<Option<GithubAuthor>> {
    map_err(nifty_github::search_user_by_email(&email, &token).map(|author| author.map(to_napi_author)))
}

#[napi]
pub fn github_lookup_user_by_email(
    email: String,
    author_map_json: String,
    token: Option<String>,
    fetch: bool,
) -> Result<Option<GithubAuthor>> {
    map_err(
        nifty_github::lookup_user_by_email(&email, &author_map_json, token.as_deref(), fetch)
            .map(|author| author.map(to_napi_author)),
    )
}
