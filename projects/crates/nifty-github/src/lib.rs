//! GitHub REST helpers for Nifty: user lookup by login or email.
//!
//! Complements [`nifty_types`] noreply parsing and `author-github.json` maps with live API calls.

mod client;
mod lookup;

pub use client::{parse_user_json, search_user_by_email, user_by_login};
pub use lookup::lookup_user_by_email;

pub use nifty_types::GithubAuthor;
