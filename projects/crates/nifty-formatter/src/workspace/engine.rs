use std::path::Path;

use oxc_formatter::JsFormatOptions;

use super::{FormatFileResult, oak, oxc, style};

/// Format JavaScript/TypeScript source: Oak AST print first, `oxc_formatter` fallback.
pub fn format_source_with_options(
    path: &Path,
    source: &str,
    options: JsFormatOptions,
) -> Result<FormatFileResult, String> {
    if let Ok(output) = oak::format_source(path, source) {
        return Ok(FormatFileResult {
            changed: output != source,
            output,
        });
    }
    oxc::format_source_with_options(path, source, options)
}

pub fn format_source(path: &Path, source: &str) -> Result<FormatFileResult, String> {
    format_source_with_options(path, source, style::default_format_options())
}
