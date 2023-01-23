use nifty_core::{format_subject, parse_subject, validate_subject, Section};

#[test]
fn parse_and_validate_gitmoji_subject() {
    let subject = "✨ Add `git-change-logs` binary";
    assert!(validate_subject(subject));
    let parsed = parse_subject(subject);
    assert_eq!(parsed.gitmoji.as_deref(), Some("✨"));
    assert_eq!(parsed.body, "Add `git-change-logs` binary");
    assert_eq!(parsed.section, Section::Features);
}

#[test]
fn format_subject_round_trip() {
    let formatted = format_subject("🐛", "Fix wasm assembly");
    assert_eq!(formatted, "🐛 Fix wasm assembly");
    assert!(validate_subject(&formatted));
}
