//! Nifty formatting: gitmoji subjects, release bullets, changelog sections, and workspace format.

mod release;
mod render;
mod subject;
mod workspace;

pub use release::{format_release_notes, format_release_section, ReleaseCommit};
pub use render::{author_mention, commit_bullet, render_section_bullets};
pub use subject::format_subject;
pub use workspace::{format_source, run_format, FormatFileResult, FormatReport, RunFormatOptions};