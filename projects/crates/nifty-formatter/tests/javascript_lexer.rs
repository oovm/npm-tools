//! Ported from `data-text-javascript/tests/lexer/mod.rs`.

use data_text_javascript::{language::JavaScriptLanguage, lexer::JavaScriptLexer};
use language_testing::lexing::LexerTester;
use std::{path::Path, time::Duration};

#[test]
fn javascript_lexer_fixtures() -> Result<(), data_text_core::TextError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = JavaScriptLanguage::standard();
    let lexer = JavaScriptLexer::new(&language);
    let test_runner = LexerTester::new(here.join("data_text_javascript/fixtures/lexer"))
        .with_extension("js")
        .with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&lexer)
}
