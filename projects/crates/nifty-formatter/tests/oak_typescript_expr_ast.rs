//! Moved from `oak-typescript/src/print/expr.rs`.

use oak_core::{Builder, ParseSession, SourceText};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, Statement},
};

fn parse_expr_snippet(trimmed: &str) -> Option<oak_typescript::ast::Expression> {
    let source = SourceText::new(trimmed);
    let language = TypeScriptLanguage::default();
    let builder = TypeScriptBuilder::new(&language);
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    built.result.ok().and_then(|root| {
        root.statements.iter().find_map(|stmt| match stmt {
            Statement::ExpressionStatement(es) => Some(es.expression.clone()),
            _ => None,
        })
    })
}

#[test]
fn prints_binary_with_spacing_via_formatter() {
    let out = oak_typescript::formatter::format_source("a+b", &oak_typescript::formatter::FormatOptions::default())
        .expect("format");
    assert_eq!(out, "a + b");
}

#[test]
fn builds_ternary_string_literals() {
    let expr = parse_expr_snippet(r#"error ? "true" : "false""#).expect("parse");
    match expr.kind.as_ref() {
        ExpressionKind::ConditionalExpression { consequent, alternate, .. } => {
            match consequent.kind.as_ref() {
                ExpressionKind::StringLiteral(s) => assert_eq!(s, "true"),
                other => panic!("consequent: {other:?}"),
            }
            match alternate.kind.as_ref() {
                ExpressionKind::StringLiteral(s) => assert_eq!(s, "false"),
                other => panic!("alternate: {other:?}"),
            }
        }
        other => panic!("root: {other:?}"),
    }
}
