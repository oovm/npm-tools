//! Moved from `oak-typescript/src/print/mod.rs` (private print path).

use oak_typescript::formatter::{FormatOptions, format_source};

#[test]
fn formats_const_spacing() {
    let out = format_source("const  x=1", &FormatOptions::default()).expect("format");
    assert_eq!(out, "const x = 1");
}

#[test]
fn format_is_idempotent() {
    let once = format_source("const x = 1", &FormatOptions::default()).expect("once");
    let twice = format_source(&once, &FormatOptions::default()).expect("twice");
    assert_eq!(once, twice);
}

#[test]
fn preserves_leading_line_comment_via_formatter() {
    let out = format_source("// keep\nconst x = 1", &FormatOptions::default()).expect("format");
    assert_eq!(out, "// keep\nconst x = 1");
}
