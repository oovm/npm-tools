//! Moved from `oak-typescript/src/formatter/cst_options.rs`.

use oak_typescript::formatter::{FormatOptions, QuoteStyle};

#[test]
fn maps_indent_and_line_width() {
    let options = FormatOptions { indent_width: 2, line_width: 100, type_erasure: false, quote_style: QuoteStyle::Double };
    assert_eq!(options.indent_width, 2);
    assert_eq!(options.line_width, 100);
    assert_eq!(options.quote_style, QuoteStyle::Double);
}
