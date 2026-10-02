use std::path::Path;

use oxc_formatter::JsFormatOptions;

use super::{FormatFileResult, oak, oxc, style};

/// Oak failures that must not fall through to legacy oxc (invalid source, unsupported syntax).
fn should_stop_at_oak(err: &str) -> bool {
    err.contains("diagnostics")
        || err.contains("parse failed")
        || err.contains("unsupported")
        || err.contains("overlapping CST spans")
}

/// Format JS/TS/JSX/TSX via Oak. Target frontend is Oak only.
pub fn format_source(path: &Path, source: &str) -> Result<FormatFileResult, String> {
    format_source_with_options(path, source, style::default_format_options())
}

/// Format JS/TS/JSX/TSX via Oak. `oxc_formatter` below is legacy debt for uncovered inputs and must shrink to zero.
pub fn format_source_with_options(
    path: &Path,
    source: &str,
    options: JsFormatOptions,
) -> Result<FormatFileResult, String> {
    let format_options = style::format_options_from_js(&options);
    match oak::format_source(path, source, &format_options) {
        Ok(output) => Ok(FormatFileResult {
            changed: output != source,
            output,
        }),
        Err(err) if should_stop_at_oak(&err) => Err(err),
        Err(_) => {
            // TODO(P4): remove once Oak `format` covers this input (remaining gaps: quote style, etc.).
            oxc::format_source_with_options(path, source, options)
        }
    }
}
