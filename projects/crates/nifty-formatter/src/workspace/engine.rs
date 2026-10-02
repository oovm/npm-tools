use std::path::Path;

use oxc_formatter::JsFormatOptions;

use super::{FormatFileResult, oak, oxc, style};

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
    if let Ok(output) = oak::format_source(path, source) {
        return Ok(FormatFileResult {
            changed: output != source,
            output,
        });
    }
    // TODO(P4): remove once Oak print covers this input (remaining gaps: class, trivia, etc.).
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
    fn cst_format_preserves_leading_line_comment() {
        let result = format_source_with_options(
            Path::new("sample.ts"),
            "// keep\nconst  x=1",
            default_format_options(),
        )
        .expect("format");
        assert_eq!(result.output, "// keep\nconst x = 1");
    }

    #[test]
    fn cst_format_preserves_trailing_comment_in_statement() {
        let input = "const x = 1 // keep";
        let result = format_source_with_options(
            Path::new("sample.ts"),
            input,
            default_format_options(),
        )
        .expect("format");
        assert_eq!(result.output, input);
    }

    #[test]
    fn cst_format_preserves_asi_sensitive_continuation() {
        let input = "const total = base\n+ extra";
        let result = format_source_with_options(
            Path::new("sample.ts"),
            input,
            default_format_options(),
        )
        .expect("format");
        assert_eq!(result.output, input);
    }

    #[test]
    fn oak_formats_jsx_via_print_path() {
        let result = format_source_with_options(
            Path::new("sample.tsx"),
            r#"const el = <div className="foo">bar</div>"#,
            default_format_options(),
        )
        .expect("format");
        assert!(
            result.output.contains("<div className='foo'>bar</div>"),
            "{}",
            result.output
        );
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
