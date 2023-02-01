//! Rule-based lint/check for Nifty gitmoji conventions.

mod context;
mod engine;
mod git;
mod rule;
mod rules;

pub use context::{CommitInput, LintContext};
pub use engine::{run_check, run_lint, LintOptions, LintReport, LintResult};
pub type Result<T> = std::result::Result<T, String>;
pub use rule::{default_rules, LintDiagnostic, RuleConfig, RuleSeverity, RULE_GITMOJI_BODY, RULE_GITMOJI_FORMAT, RULE_GITMOJI_KNOWN, RULE_GITMOJI_SUBJECT};
