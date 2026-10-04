//! Shared Oak `format` contract through the Nifty workspace adapter.
//! Conformance matrix: `projects/packages/nifty-skills` Oak formatter fixtures and this test suite.

use std::path::Path;

use nifty_formatter::{default_format_options, format_source_with_options};

fn format_sample(path: &str, input: &str) -> String {
    format_source_with_options(Path::new(path), input, default_format_options())
        .unwrap_or_else(|err| panic!("format failed: {err}"))
        .output
}

#[test]
fn preserves_leading_line_comment_and_normalizes_const() {
    let input = "// keep\nconst  x=1";
    let out = format_sample("sample.ts", input);
    assert_eq!(out, "// keep\nconst x = 1");
    let again = format_sample("sample.ts", &out);
    assert_eq!(out, again);
}

#[test]
fn formats_top_level_class_without_ast_fallback() {
    let output = format_sample("sample.ts", "class Foo{}");
    assert_eq!(output, "class Foo { }");
}

#[test]
fn formats_nifty_native_import_header() {
    let source = r#"import type { CommitRecord } from "./types.js";"#;
    format_sample("native.ts", source);
}

#[test]
fn formats_optional_property_spacing() {
    let source = include_str!("../../../packages/nifty/src/cli/authArgs.ts");
    let output = format_sample("authArgs.ts", source);
    assert!(output.contains("otp?: string;"), "output={output:?}");
    assert!(!output.contains("otp ? :"), "output={output:?}");
}

#[test]
fn formats_comparisons_nullish_coalescing_and_ternary_object_values() {
    let input = "const value=items.length>0?items[0]:fallback; const roots={cargoRoot:config.cargoRoot??layout.root};";
    let output = format_sample("sample.ts", input);
    assert_eq!(
        output,
        "const value = items.length > 0 ? items[0] : fallback; const roots = { cargoRoot: config.cargoRoot ?? layout.root };"
    );
    assert_eq!(format_sample("sample.ts", &output), output);
}

#[test]
fn formats_release_cli_with_regex_literals_without_changing_tokens() {
    let source = include_str!("../../../packages/nifty/src/cli/bump.ts");
    let output = format_sample("bump.ts", source);
    assert_eq!(format_sample("bump.ts", &output), output);
    assert!(output.contains(r#"/^version\s*=\s*"([^"]*)"/m"#));
    assert!(output.contains(r#"/^\d+\.\d+\.\d+(-[\w.-]+)?$/"#));
}

#[test]
fn preserves_regex_bodies_and_distinguishes_division() {
    for source in [
        r#"const pattern=/[{}()\[\]'"/]+/gu;"#,
        r#"const pattern=/a\/b/; const ratio = left / right / scale;"#,
        r#"function matches(text: string) { return /["}]/.test(text); }"#,
    ] {
        let output = format_sample("regex.ts", source);
        assert_eq!(format_sample("regex.ts", &output), output);
        for literal in [r#"/[{}()\[\]'"/]+/gu"#, r#"/a\/b/"#, r#"/["}]/"#] {
            if source.contains(literal) {
                assert!(output.contains(literal), "output={output:?}");
            }
        }
    }
}

#[test]
fn workspace_typescript_formatting_preserves_every_token() {
    use oak_core::{Lexer, ParseSession, SourceText};
    use oak_typescript::{TypeScriptLanguage, TypeScriptLexer, TypeScriptTokenType};

    let language = TypeScriptLanguage::default();
    let lexer = TypeScriptLexer::new(&language);
    let token_texts = |source: &str| {
        let mut session = ParseSession::default();
        let text = SourceText::new(source);
        lexer
            .lex(&text, &[], &mut session)
            .result
            .unwrap()
            .iter()
            .filter(|token| !matches!(token.kind, TypeScriptTokenType::Whitespace | TypeScriptTokenType::Newline))
            .map(|token| (token.kind, source[token.span.start..token.span.end].to_owned()))
            .collect::<Vec<_>>()
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/nifty/src");
    let mut failures = Vec::new();
    for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("ts") {
            continue;
        }
        let source = std::fs::read_to_string(path).unwrap();
        match format_source_with_options(path, &source, default_format_options()) {
            Ok(result) => {
                if token_texts(&source) != token_texts(&result.output) {
                    failures.push(format!("{}: token stream changed", path.display()));
                }
                if format_sample("sample.ts", &result.output) != result.output {
                    failures.push(format!("{}: format is not idempotent", path.display()));
                }
            }
            Err(error) => failures.push(format!("{}: {error}", path.display())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn preserves_trailing_comment_and_block_comment_in_statement() {
    let input = "const x = 1 /* mid */ // end";
    assert_eq!(format_sample("sample.ts", input), input);
}

#[test]
fn preserves_asi_sensitive_continuation_line() {
    let input = "const total = base\n+ extra";
    assert_eq!(format_sample("sample.ts", input), input);
}

#[test]
fn preserves_block_comment_between_statements() {
    let input = "const a = 1\n/* between */\nconst b = 2";
    assert_eq!(format_sample("sample.ts", input), "const a = 1\n/* between */\nconst b = 2");
}

#[test]
fn formats_scoped_import_without_comments() {
    let input = "import  {  foo }  from 'pkg'";
    assert_eq!(format_sample("sample.ts", input), "import { foo } from 'pkg'");
}

#[test]
fn rejects_unclosed_brace_without_rewrite() {
    let err = format_source_with_options(Path::new("sample.ts"), "const x = {", default_format_options()).unwrap_err();
    assert!(err.contains("unbalanced delimiters"), "err={err}");
}

#[test]
fn preserves_decorated_const_statement() {
    let input = "@Component()\nconst  x=1";
    assert_eq!(format_sample("sample.ts", input), "@Component()\nconst x = 1");
}

#[test]
fn ternary_string_literals_format_and_idempotent() {
    let input = r#"const v = error ? "true" : "false""#;
    let out = format_sample("sample.ts", input);
    assert_eq!(out, "const v = error ? \"true\" : \"false\"");
    assert_eq!(format_sample("sample.ts", &out), out);
}

#[test]
fn jsx_cases_format_and_idempotent() {
    let cases = [
        (r#"const el = <div className="foo">bar</div>"#, None::<&str>),
        ("const el = <br />", Some("const el = <br />")),
        ("const el = <>hello</>", Some("const el = <>hello</>")),
    ];
    for (input, expected) in cases {
        let out = format_sample("sample.tsx", input);
        if let Some(expected) = expected {
            assert_eq!(out, expected, "input={input:?}");
        }
        else {
            assert!(out.contains("<div className = \"foo\">bar</div>"), "out={out:?}");
        }
        assert_eq!(format_sample("sample.tsx", &out), out, "input={input:?}");
    }
}
