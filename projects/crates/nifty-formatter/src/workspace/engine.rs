use std::path::Path;

use oxc_formatter::JsFormatOptions;

use super::{FormatFileResult, oak, oxc, style};

/// Format JavaScript/TypeScript/JSX/TSX: Oak AST print first, `oxc_formatter` fallback.
pub fn format_source(path: &Path, source: &str) -> Result<FormatFileResult, String> {
    format_source_with_options(path, source, style::default_format_options())
}

/// Format JavaScript/TypeScript/JSX/TSX: Oak AST print first, `oxc_formatter` fallback.
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::workspace::style::{default_format_options, resolve_format_options, FormatStyleOptions};

    #[test]
    fn oak_formats_typescript_source() {
        let result = format_source_with_options(
            Path::new("sample.ts"),
            "const  x=1",
            default_format_options(),
        )
        .expect("format");
        assert!(result.changed);
        assert!(result.output.contains("const x = 1"), "{}", result.output);
    }

    #[test]
    fn oak_format_is_idempotent_for_simple_ts() {
        let once = format_source_with_options(
            Path::new("sample.ts"),
            "const x = 1\n",
            default_format_options(),
        )
        .expect("once");
        let twice = format_source_with_options(
            Path::new("sample.ts"),
            &once.output,
            default_format_options(),
        )
        .expect("twice");
        assert_eq!(once.output, twice.output);
    }

    #[test]
    fn resolve_style_still_applies_to_oxc_fallback() {
        let options = resolve_format_options(Some(&FormatStyleOptions {
            indent_style: Some("space".to_string()),
            indent_width: Some(2),
            line_width: Some(100),
            quote_style: Some("double".to_string()),
        }));
        assert_eq!(options.indent_width.value(), 2);
        assert_eq!(options.line_width.value(), 100);
    }
}
