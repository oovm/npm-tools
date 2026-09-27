use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter::{format, JsFormatOptions};
use oxc_span::SourceType;

use super::style::default_format_options;
use crate::workspace::FormatFileResult;

pub fn format_source_with_options(
    path: &Path,
    source: &str,
    options: JsFormatOptions,
) -> Result<FormatFileResult, String> {
    let source_type = SourceType::from_path(path)
        .map_err(|err| format!("{}: {err}", path.display()))?;
    let allocator = Allocator::new();
    let formatted = format(&allocator, source, source_type, options)
        .map_err(|err| format!("{}: {err}", path.display()))?;
    let code = formatted
        .print()
        .map_err(|err| format!("{}: print error: {err}", path.display()))?
        .into_code();

    Ok(FormatFileResult {
        changed: code != source,
        output: code,
    })
}

pub fn format_source(path: &Path, source: &str) -> Result<FormatFileResult, String> {
    format_source_with_options(path, source, default_format_options())
}
