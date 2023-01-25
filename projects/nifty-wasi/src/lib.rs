//! WASI Preview 2 export surface for Nifty (gitmoji + gix git helpers).

#![cfg_attr(all(target_os = "wasi", target_env = "p2"), feature(wasip2))]

use std::path::Path;

use nifty_core::{
    GithubAuthor as CoreAuthor, author_mention, avatar_url, commit_bullet, display_login, format_subject,
    github_from_noreply_email, known_gitmojis, leading_gitmoji, load_author_map_from_json, parse_subject,
    profile_url, resolve_github_author, section_for_gitmoji, section_name, strip_gitmoji, validate_subject,
};
use nifty_git::{
    CommitRecord as CoreCommit, RangeInfo as CoreRange, TagInfo as CoreTag, collect_commits, detect_github_repo,
    discover_root, format_tag_list, list_tag_infos, list_version_tags, parse_github_remote_repo, resolve_range,
};

wit_bindgen::generate!({
    world: "nifty",
    path: "wit",
});

use exports::doki::land::git::{CommitRecord, RangeInfo, TagInfo};
use exports::doki::land::gitmoji::{GithubAuthor, ParsedSubject};

struct Nifty;

fn to_wit_author(author: CoreAuthor) -> GithubAuthor {
    GithubAuthor { id: author.id, login: author.login }
}

fn from_wit_author(author: GithubAuthor) -> CoreAuthor {
    CoreAuthor { id: author.id, login: author.login }
}

fn to_wit_parsed(parsed: nifty_core::ParsedSubject) -> ParsedSubject {
    ParsedSubject {
        gitmoji: parsed.gitmoji,
        body: parsed.body,
        section: section_name(parsed.section).to_string(),
    }
}

fn load_map(json: &str) -> std::collections::HashMap<String, CoreAuthor> {
    load_author_map_from_json(json)
}

fn to_wit_tag(tag: CoreTag) -> TagInfo {
    TagInfo { name: tag.name, short_hash: tag.short_hash }
}

fn to_wit_range(range: CoreRange) -> RangeInfo {
    RangeInfo {
        version: range.version,
        from_ref: range.from_ref,
        to_ref: range.to_ref,
    }
}

fn to_wit_commit(commit: CoreCommit) -> CommitRecord {
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

impl exports::doki::land::gitmoji::Guest for Nifty {
    fn known_gitmojis() -> Vec<String> {
        known_gitmojis().iter().map(|emoji| emoji.to_string()).collect()
    }

    fn validate_subject(subject: String) -> bool {
        validate_subject(&subject)
    }

    fn parse_subject(subject: String) -> ParsedSubject {
        to_wit_parsed(parse_subject(&subject))
    }

    fn format_subject(gitmoji: String, body: String) -> String {
        format_subject(&gitmoji, &body)
    }

    fn strip_gitmoji(subject: String) -> String {
        strip_gitmoji(&subject)
    }

    fn leading_gitmoji(subject: String) -> Option<String> {
        leading_gitmoji(&subject).map(str::to_string)
    }

    fn section_for_gitmoji(gitmoji: Option<String>) -> String {
        section_name(section_for_gitmoji(gitmoji.as_deref())).to_string()
    }

    fn parse_noreply_email(email: String) -> Option<GithubAuthor> {
        github_from_noreply_email(&email).map(to_wit_author)
    }

    fn resolve_github_author(email: String, author_map_json: String) -> Option<GithubAuthor> {
        let map = load_map(&author_map_json);
        resolve_github_author(&email, &map).map(to_wit_author)
    }

    fn display_login(author: GithubAuthor) -> String {
        display_login(&from_wit_author(author))
    }

    fn profile_url(author: GithubAuthor) -> String {
        profile_url(&from_wit_author(author))
    }

    fn avatar_url(author: GithubAuthor) -> String {
        avatar_url(&from_wit_author(author))
    }

    fn author_mention(email: String, author_name: String, author_map_json: String) -> String {
        let map = load_map(&author_map_json);
        author_mention(&email, &author_name, &map)
    }

    fn commit_bullet(body: String, email: String, author_name: String, author_map_json: String) -> String {
        let map = load_map(&author_map_json);
        commit_bullet(&body, &email, &author_name, &map)
    }
}

#[cfg(not(target_os = "wasi"))]
impl exports::doki::land::github::Guest for Nifty {
    fn user_by_login(login: String, token: Option<String>) -> Result<GithubAuthor, String> {
        nifty_github::user_by_login(&login, token.as_deref()).map(to_wit_author)
    }

    fn search_user_by_email(email: String, token: String) -> Result<Option<GithubAuthor>, String> {
        nifty_github::search_user_by_email(&email, &token)
            .map(|author| author.map(to_wit_author))
    }

    fn lookup_user_by_email(
        email: String,
        author_map_json: String,
        token: Option<String>,
        fetch: bool,
    ) -> Result<Option<GithubAuthor>, String> {
        nifty_github::lookup_user_by_email(&email, &author_map_json, token.as_deref(), fetch)
            .map(|author| author.map(to_wit_author))
    }
}

#[cfg(target_os = "wasi")]
impl exports::doki::land::github::Guest for Nifty {
    fn user_by_login(_login: String, _token: Option<String>) -> Result<GithubAuthor, String> {
        Err("GitHub API is unavailable in the WASI component".to_string())
    }

    fn search_user_by_email(_email: String, _token: String) -> Result<Option<GithubAuthor>, String> {
        Err("GitHub API is unavailable in the WASI component".to_string())
    }

    fn lookup_user_by_email(
        _email: String,
        _author_map_json: String,
        _token: Option<String>,
        _fetch: bool,
    ) -> Result<Option<GithubAuthor>, String> {
        Err("GitHub API is unavailable in the WASI component".to_string())
    }
}

impl exports::doki::land::git::Guest for Nifty {
    fn discover_root(start_path: String) -> Result<String, String> {
        discover_root(Path::new(&start_path)).map(|path| path.display().to_string())
    }

    fn list_version_tags(repo_root: String) -> Result<Vec<String>, String> {
        list_version_tags(Path::new(&repo_root))
    }

    fn list_tag_infos(repo_root: String) -> Result<Vec<TagInfo>, String> {
        list_tag_infos(Path::new(&repo_root)).map(|tags| tags.into_iter().map(to_wit_tag).collect())
    }

    fn format_tag_list(repo_root: String) -> Result<String, String> {
        format_tag_list(Path::new(&repo_root))
    }

    fn resolve_range(
        repo_root: String,
        version: Option<String>,
        from_ref: Option<String>,
        to_ref: Option<String>,
    ) -> Result<RangeInfo, String> {
        resolve_range(
            Path::new(&repo_root),
            version.as_deref(),
            from_ref.as_deref(),
            to_ref.as_deref(),
        )
        .map(to_wit_range)
    }

    fn collect_commits(repo_root: String, from_ref: Option<String>, to_ref: String) -> Result<Vec<CommitRecord>, String> {
        collect_commits(
            Path::new(&repo_root),
            from_ref.as_deref(),
            &to_ref,
        )
        .map(|commits| commits.into_iter().map(to_wit_commit).collect())
    }

    fn detect_github_repo(repo_root: String) -> Result<Option<String>, String> {
        detect_github_repo(Path::new(&repo_root))
    }

    fn parse_github_remote(url: String) -> Option<String> {
        parse_github_remote_repo(&url)
    }
}

export!(Nifty);
