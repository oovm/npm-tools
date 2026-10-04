//! Nifty formatting: gitmoji subjects, release bullets, changelog sections, and workspace format.

mod release;
mod render;
mod subject;
mod workspace;

pub use release::{ReleaseCommit, format_release_notes, format_release_section};
pub use render::{author_mention, commit_bullet, render_section_bullets};
pub use subject::format_subject;
pub use workspace::{
    FormatFileResult, FormatReport, FormatStyleOptions, RunFormatOptions, default_format_options, format_source,
    format_source_with_options, resolve_format_options, run_format,
};
