use std::path::Path;

use data_text_typescript::formatter::FormatOptions;

use super::{FormatFileResult, text};

/// Format JavaScript and TypeScript through `data.text` without a legacy fallback.
pub fn format_source(path: &Path, source: &str) -> Result<FormatFileResult, String> {
    format_source_with_options(path, source, super::style::default_format_options())
}

/// Format JavaScript and TypeScript through the `data.text` public formatter contract.
pub fn format_source_with_options(path: &Path, source: &str, options: FormatOptions) -> Result<FormatFileResult, String> {
    let output = text::format_source(path, source, &options)?;
    Ok(FormatFileResult { changed: output != source, output })
}
