use nifty_linter::{LintOptions, RULE_GITMOJI_SUBJECT, run_lint};

#[test]
fn lints_invalid_subject_via_cli_options() {
    let report = run_lint(LintOptions { subjects: vec!["Add feature".to_string()], ..Default::default() }).expect("lint");
    assert!(report.diagnostics.iter().any(|item| item.rule == RULE_GITMOJI_SUBJECT));
}

#[test]
fn accepts_valid_subject() {
    let report = run_lint(LintOptions { subjects: vec!["✨ Add feature".to_string()], ..Default::default() }).expect("lint");
    assert!(report.diagnostics.is_empty());
}
