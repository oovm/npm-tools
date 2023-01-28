use nifty_formatter::{author_mention, commit_bullet, format_release_notes, format_subject, ReleaseCommit};
use nifty_types::Section;

#[test]
fn format_subject_round_trip() {
    let formatted = format_subject("🐛", "Fix wasm assembly");
    assert_eq!(formatted, "🐛 Fix wasm assembly");
}

#[test]
fn release_notes_include_section_heading() {
    let commits = [ReleaseCommit {
        body: "Add feature",
        email: "dev@example.com",
        author: "Dev",
        section: Section::Features,
    }];
    let notes = format_release_notes(&commits, &Default::default());
    assert!(notes.contains("## ✨ Features"));
    assert!(notes.contains("- Add feature (@Dev)"));
}

#[test]
fn author_mention_falls_back_to_name() {
    let mention = author_mention("x@y.com", "Alice Bob", &Default::default());
    assert_eq!(mention, "@AliceBob");
}
