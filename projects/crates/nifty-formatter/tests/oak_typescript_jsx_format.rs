//! Moved from `oak-typescript/src/print/jsx.rs` — formatter contract for JSX snippets.

use oak_typescript::formatter::{FormatOptions, format_source};

#[test]
fn jsx_element_with_string_attribute_formats() {
    let input = r#"const x = <div className="foo">bar</div>"#;
    let out = format_source(input, &FormatOptions::default()).expect("format");
    assert_eq!(out, r#"const x = <div className="foo">bar</div>"#);
}

#[test]
fn jsx_self_closing_formats() {
    let input = "const x = <br />";
    assert_eq!(format_source(input, &FormatOptions::default()).expect("format"), input);
}

#[test]
fn jsx_fragment_formats() {
    let input = "const x = <>hello</>";
    assert_eq!(format_source(input, &FormatOptions::default()).expect("format"), input);
}
