use crate::rule::{LintDiagnostic, RuleConfig, RuleSeverity};

/// One commit subject to lint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInput {
    pub hash: Option<String>,
    pub subject: String,
}

/// Inputs and enabled rules for linting.
#[derive(Debug, Clone)]
pub struct LintContext {
    pub commits: Vec<CommitInput>,
    pub rules: Vec<RuleConfig>,
}

impl LintContext {
    pub fn new(commits: Vec<CommitInput>, rules: Vec<RuleConfig>) -> Self {
        Self { commits, rules }
    }

    pub fn severity_for(&self, rule_id: &str) -> Option<RuleSeverity> {
        self.rules
            .iter()
            .find(|rule| rule.id == rule_id && rule.enabled)
            .map(|rule| rule.severity)
    }
}

impl CommitInput {
    pub fn subject_only(subject: impl Into<String>) -> Self {
        Self {
            hash: None,
            subject: subject.into(),
        }
    }
}

pub fn diagnostic(
    ctx: &LintContext,
    rule_id: &'static str,
    message: impl Into<String>,
    commit: &CommitInput,
) -> Option<LintDiagnostic> {
    let severity = ctx.severity_for(rule_id)?;
    Some(LintDiagnostic {
        rule: rule_id.to_string(),
        severity,
        message: message.into(),
        subject: Some(commit.subject.clone()),
        hash: commit.hash.clone(),
    })
}
