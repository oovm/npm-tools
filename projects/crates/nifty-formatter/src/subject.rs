//! Gitmoji subject formatting.

/// Format a commit subject as `<gitmoji> <body>` (Nifty default convention).
pub fn format_subject(gitmoji: &str, body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        gitmoji.to_string()
    } else {
        format!("{gitmoji} {body}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_subject;
    use nifty_types::validate_subject;

    #[test]
    fn formats_gitmoji_subject() {
        let formatted = format_subject("🐛", "Fix wasm assembly");
        assert_eq!(formatted, "🐛 Fix wasm assembly");
        assert!(validate_subject(&formatted));
    }
}
