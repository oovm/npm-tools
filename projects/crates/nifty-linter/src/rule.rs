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
    ]
}

impl LintDiagnostic {
    pub fn is_error(&self) -> bool {
        self.severity == RuleSeverity::Error
    }
}
