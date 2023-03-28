//! Rule-based lint/check for Nifty gitmoji conventions and cargo workspace hygiene.

mod cargo;
mod context;
mod engine;
mod git;
mod rule;
mod rules;

pub use context::{CommitInput, LintContext};
pub use engine::{run_check, run_lint, LintOptions, LintReport, LintResult};
pub type Result<T> = std::result::Result<T, String>;
pub use rule::{
    default_rules, LintDiagnostic, RuleConfig, RuleSeverity, RULE_CARGO_DOC_INCLUDE_STR, RULE_CARGO_LARGE_FILE,
    RULE_CARGO_MISPLACED_ROOT_RS, RULE_CARGO_MISPLACED_TEST, RULE_CARGO_MISSING_DOCS, RULE_CARGO_PACKAGE_SECTION,
    RULE_CARGO_README_CASE, RULE_CARGO_README_MISSING, RULE_CARGO_WORKSPACE_DEP, RULE_CARGO_WORKSPACE_INHERIT,
    RULE_GITMOJI_BODY, RULE_GITMOJI_FORMAT, RULE_GITMOJI_KNOWN, RULE_GITMOJI_SUBJECT,
};
