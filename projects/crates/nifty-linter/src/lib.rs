//! Rule-based lint/check for Nifty gitmoji conventions and cargo workspace hygiene.

mod cargo;
mod context;
mod engine;
mod git;
mod rule;
mod rules;
mod workspace;

pub use context::{CommitInput, LintContext};
pub use engine::{LintOptions, LintReport, LintResult, run_check, run_lint};
pub type Result<T> = std::result::Result<T, String>;
pub use rule::{
    LintDiagnostic, RULE_CARGO_DOC_INCLUDE_STR, RULE_CARGO_LARGE_FILE, RULE_CARGO_MISPLACED_ROOT_RS, RULE_CARGO_MISPLACED_TEST,
    RULE_CARGO_MISSING_DOCS, RULE_CARGO_PACKAGE_SECTION, RULE_CARGO_README_CASE, RULE_CARGO_README_MISSING,
    RULE_CARGO_WORKSPACE_DEP, RULE_CARGO_WORKSPACE_INHERIT, RULE_COMMIT_BARE_PACKAGE, RULE_COMMIT_BARE_SYMBOL,
    RULE_COMMIT_SEMICOLON, RULE_COMMIT_SEMVER, RULE_GITMOJI_BODY, RULE_GITMOJI_FORMAT, RULE_GITMOJI_KNOWN,
    RULE_GITMOJI_SUBJECT, RuleConfig, RuleSeverity, default_commit_rules, default_rules,
};
