use nifty_types::{parse_subject, validate_subject, Section};

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
fn accepts_init_gitmoji_subject() {
    let subject = "🎂 Project initialized!";
    assert!(validate_subject(subject));
    let parsed = parse_subject(subject);
    assert_eq!(parsed.gitmoji.as_deref(), Some("🎂"));
    assert_eq!(parsed.section, Section::Other);
}

#[test]
fn accepts_pin_gitmoji_subject() {
    let subject = "📌 Pin homepage to published `@notedge/panduck` `0.0.2`";
    assert!(validate_subject(subject));
    let parsed = parse_subject(subject);
    assert_eq!(parsed.gitmoji.as_deref(), Some("📌"));
    assert_eq!(parsed.section, Section::Other);
}
