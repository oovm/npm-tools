//! Moved from `oak-javascript/tests/lexer/mod.rs`.

use oak_javascript::{language::JavaScriptLanguage, lexer::JavaScriptLexer};
use oak_testing::lexing::LexerTester;
use std::{path::Path, time::Duration};

#[test]
fn javascript_lexer_fixtures() -> Result<(), oak_core::OakError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = JavaScriptLanguage::standard();
    let lexer = JavaScriptLexer::new(&language);
    let test_runner = LexerTester::new(here.join("oak_javascript/fixtures/lexer"))
        .with_extension("js")
        .with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&lexer)
}
