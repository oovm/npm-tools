//! Moved from `oak-typescript/src/formatter/red_tree/mod.rs` — public formatter entry only.

use oak_typescript::formatter::{FormatOptions, format_source};

#[test]
fn variable_declaration_rule_normalizes_spacing() {
    assert_eq!(format_source("const  x=1", &FormatOptions::default()).expect("format"), "const x = 1");
}

#[test]
fn import_declaration_rule_normalizes_spacing() {
    assert_eq!(
        format_source("import  {  foo }  from 'pkg'", &FormatOptions::default()).expect("format"),
        "import { foo } from 'pkg'"
    );
}
