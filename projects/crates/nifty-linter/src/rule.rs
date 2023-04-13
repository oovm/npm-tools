use serde::{Deserialize, Serialize};

/// Severity for a lint rule violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}

/// One lint finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintDiagnostic {
    pub rule: String,
    pub severity: RuleSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Rule toggle and severity from config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleConfig {
    pub id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_severity")]
    pub severity: RuleSeverity,
}

pub const RULE_GITMOJI_SUBJECT: &str = "gitmoji/subject";
pub const RULE_GITMOJI_KNOWN: &str = "gitmoji/known";
pub const RULE_GITMOJI_BODY: &str = "gitmoji/body";
pub const RULE_GITMOJI_FORMAT: &str = "gitmoji/format";

pub const RULE_COMMIT_SEMVER: &str = "commit/semver";
pub const RULE_COMMIT_SEMICOLON: &str = "commit/semicolon";
pub const RULE_COMMIT_BARE_PACKAGE: &str = "commit/bare-package";
pub const RULE_COMMIT_BARE_SYMBOL: &str = "commit/bare-symbol";

pub const RULE_CARGO_README_CASE: &str = "cargo/readme-case";
pub const RULE_CARGO_PACKAGE_SECTION: &str = "cargo/package-section";
pub const RULE_CARGO_README_MISSING: &str = "cargo/readme-missing";
pub const RULE_CARGO_MISSING_DOCS: &str = "cargo/missing-docs";
pub const RULE_CARGO_WORKSPACE_INHERIT: &str = "cargo/workspace-inherit";
pub const RULE_CARGO_WORKSPACE_DEP: &str = "cargo/workspace-dep";
pub const RULE_CARGO_DOC_INCLUDE_STR: &str = "cargo/doc-include-str";
pub const RULE_CARGO_MISPLACED_TEST: &str = "cargo/misplaced-test";
pub const RULE_CARGO_MISPLACED_ROOT_RS: &str = "cargo/misplaced-root-rs";
pub const RULE_CARGO_LARGE_FILE: &str = "cargo/large-file";

fn default_enabled() -> bool {
    true
}

fn default_severity() -> RuleSeverity {
    RuleSeverity::Error
}

pub fn default_rules() -> Vec<RuleConfig> {
    vec![
        RuleConfig {
            id: RULE_GITMOJI_SUBJECT.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_GITMOJI_KNOWN.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_GITMOJI_BODY.to_string(),
            enabled: true,
            severity: RuleSeverity::Warning,
        },
        RuleConfig {
            id: RULE_GITMOJI_FORMAT.to_string(),
            enabled: true,
            severity: RuleSeverity::Warning,
        },
        RuleConfig {
            id: RULE_CARGO_README_CASE.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_PACKAGE_SECTION.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_README_MISSING.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_MISSING_DOCS.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_WORKSPACE_INHERIT.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_WORKSPACE_DEP.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_DOC_INCLUDE_STR.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_MISPLACED_TEST.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_MISPLACED_ROOT_RS.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_CARGO_LARGE_FILE.to_string(),
            enabled: true,
            severity: RuleSeverity::Warning,
        },
    ]
}

pub fn default_commit_rules() -> Vec<RuleConfig> {
    vec![
        RuleConfig {
            id: RULE_GITMOJI_SUBJECT.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_GITMOJI_KNOWN.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_GITMOJI_BODY.to_string(),
            enabled: true,
            severity: RuleSeverity::Warning,
        },
        RuleConfig {
            id: RULE_GITMOJI_FORMAT.to_string(),
            enabled: true,
            severity: RuleSeverity::Warning,
        },
        RuleConfig {
            id: RULE_COMMIT_SEMVER.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_COMMIT_SEMICOLON.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_COMMIT_BARE_PACKAGE.to_string(),
            enabled: true,
            severity: RuleSeverity::Error,
        },
        RuleConfig {
            id: RULE_COMMIT_BARE_SYMBOL.to_string(),
            enabled: true,
            severity: RuleSeverity::Warning,
        },
    ]
}

impl LintDiagnostic {
    pub fn is_error(&self) -> bool {
        self.severity == RuleSeverity::Error
    }
}
