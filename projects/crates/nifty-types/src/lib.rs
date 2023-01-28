//! Nifty shared types: **gitmoji-first** commit conventions, GitHub authors, and release reference helpers.

mod author;
mod gitmoji;

pub use author::{
    GithubAuthor, avatar_url, display_login, github_from_noreply_email, load_author_map_from_json, merge_authors,
    parse_author_entry, profile_url, resolve_github_author,
};
pub use gitmoji::{
    ParsedSubject, Section, all_sections, known_gitmojis, leading_gitmoji, parse_subject,
    section_for_gitmoji, section_from_name, section_meta, section_name, strip_gitmoji, validate_subject,
};
