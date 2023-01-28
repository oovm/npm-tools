//! Nifty formatting: gitmoji subjects, release bullets, and changelog sections.

mod release;
mod render;
mod subject;

pub use release::{format_release_notes, format_release_section, ReleaseCommit};
pub use render::{author_mention, commit_bullet, render_section_bullets};
pub use subject::format_subject;
