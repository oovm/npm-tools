mod cargo;

use nifty_formatter::format_subject;
use nifty_types::{known_gitmojis, leading_gitmoji, parse_subject, strip_gitmoji, validate_subject};

use crate::context::{diagnostic, CommitInput, LintContext};
use crate::rule::{
    LintDiagnostic, RULE_GITMOJI_BODY, RULE_GITMOJI_FORMAT, RULE_GITMOJI_KNOWN, RULE_GITMOJI_SUBJECT,
};

pub use cargo::lint_cargo_workspace;
pub fn lint_commit(ctx: &LintContext, commit: &CommitInput) -> Vec<LintDiagnostic> {
    let mut out = Vec::new();
    for rule in [
        check_gitmoji_subject,
        check_gitmoji_known,
        check_gitmoji_body,
        check_gitmoji_format,
    ] {
        if let Some(item) = rule(ctx, commit) {
            out.push(item);
        }
    }
    out
}

fn check_gitmoji_subject(ctx: &LintContext, commit: &CommitInput) -> Option<LintDiagnostic> {
    if validate_subject(&commit.subject) {
        return None;
    }
    diagnostic(
        ctx,
        RULE_GITMOJI_SUBJECT,
        "commit subject must start with a known gitmoji followed by a space",
        commit,
    )
}

fn check_gitmoji_known(ctx: &LintContext, commit: &CommitInput) -> Option<LintDiagnostic> {
    let Some(emoji) = leading_gitmoji(&commit.subject) else {
        return None;
    };
    if known_gitmojis().contains(&emoji) {
        return None;
    }
    diagnostic(
        ctx,
        RULE_GITMOJI_KNOWN,
        format!("gitmoji `{emoji}` is not in the Nifty known gitmoji list"),
        commit,
    )
}

fn check_gitmoji_body(ctx: &LintContext, commit: &CommitInput) -> Option<LintDiagnostic> {
    if !validate_subject(&commit.subject) {
        return None;
    }
    let body = strip_gitmoji(&commit.subject);
    if body.trim().is_empty() {
        return diagnostic(
            ctx,
            RULE_GITMOJI_BODY,
            "commit subject body must not be empty after the gitmoji prefix",
            commit,
        );
    }
    None
}

fn check_gitmoji_format(ctx: &LintContext, commit: &CommitInput) -> Option<LintDiagnostic> {
    if !validate_subject(&commit.subject) {
        return None;
    }
    let parsed = parse_subject(&commit.subject);
    let Some(gitmoji) = parsed.gitmoji.as_deref() else {
        return None;
    };
    let expected = format_subject(gitmoji, &parsed.body);
    if expected == commit.subject.trim() {
        return None;
    }
    diagnostic(
        ctx,
        RULE_GITMOJI_FORMAT,
        format!("subject should be formatted as `{expected}`"),
        commit,
    )
}

#[cfg(test)]
mod tests {
    use super::lint_commit;
    use crate::context::{CommitInput, LintContext};
    use crate::rule::default_rules;

    #[test]
    fn flags_missing_gitmoji_subject() {
        let ctx = LintContext::new(vec![], default_rules());
        let commit = CommitInput::subject_only("Add feature");
        let diagnostics = lint_commit(&ctx, &commit);
        assert!(diagnostics.iter().any(|item| item.rule == "gitmoji/subject"));
    }

    #[test]
    fn accepts_valid_gitmoji_subject() {
        let ctx = LintContext::new(vec![], default_rules());
        let commit = CommitInput::subject_only("✨ Add feature");
        let diagnostics = lint_commit(&ctx, &commit);
        assert!(diagnostics.is_empty());
    }
}
