use std::path::Path;

use nifty_formatter::{author_mention, format_release_notes, format_source, format_subject, ReleaseCommit};
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

#[test]
fn oak_formats_typescript_source() {
    let result = format_source(Path::new("sample.ts"), "const  x=1").expect("format");
    assert!(result.changed);
    assert!(result.output.contains("const x = 1"));
}

#[test]
fn oak_formats_jsx_source() {
    let result = format_source(
        Path::new("sample.tsx"),
        r#"const el = <div className="foo">bar</div>"#,
    )
    .expect("format");
    assert!(result.output.contains("<div className='foo'>bar</div>"));
}

#[test]
fn oak_format_is_idempotent_for_simple_ts() {
    let once = format_source(Path::new("sample.ts"), "const x = 1\n").expect("once");
    let twice = format_source(Path::new("sample.ts"), &once.output).expect("twice");
    assert_eq!(once.output, twice.output);
}
