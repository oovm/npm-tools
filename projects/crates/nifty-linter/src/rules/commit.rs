use crate::{
    context::{CommitInput, LintContext, diagnostic},
    rule::{LintDiagnostic, RULE_COMMIT_BARE_PACKAGE, RULE_COMMIT_BARE_SYMBOL, RULE_COMMIT_SEMICOLON, RULE_COMMIT_SEMVER},
};

const BARE_SYMBOLS: &[&str] = &[
    "Button",
    "DatePicker",
    "Drawer",
    "Empty",
    "Alert",
    "Dialog",
    "Form",
    "Card",
    "Link",
    "RouteId",
    "SSR",
    "BindingId",
    "UserCard",
    "IndexPage",
    "VMZ",
    "Biome",
    "HTML",
    "ESM",
    "N-API",
    "Rust",
    "Node",
    "pnpm",
    "CI",
    "nifty",
    "Octokit",
    "octocrab",
    "AST",
    "HMR",
    "DevInput",
    "CLI",
    "TS",
];

pub fn lint_commit_hygiene(ctx: &LintContext, commit: &CommitInput) -> Vec<LintDiagnostic> {
    let message = commit.full_message();
    let mut out = Vec::new();
    for rule in [check_semver, check_semicolon, check_bare_package, check_bare_symbols] {
        if let Some(item) = rule(ctx, commit, message) {
            out.push(item);
        }
    }
    out
}

fn check_semver(ctx: &LintContext, commit: &CommitInput, message: &str) -> Option<LintDiagnostic> {
    let subject = commit.subject.trim();
    if let Some(found) = find_semver(subject) {
        return diagnostic(ctx, RULE_COMMIT_SEMVER, format!("semver `{found}` must not appear in the commit subject"), commit);
    }
    if let Some(found) = find_semver(message) {
        return diagnostic(ctx, RULE_COMMIT_SEMVER, format!("semver `{found}` must not appear in the commit message"), commit);
    }
    None
}

fn check_semicolon(ctx: &LintContext, commit: &CommitInput, message: &str) -> Option<LintDiagnostic> {
    if message.contains(';') || message.contains('；') {
        return diagnostic(ctx, RULE_COMMIT_SEMICOLON, "commit message must not contain semicolons", commit);
    }
    None
}

fn check_bare_package(ctx: &LintContext, commit: &CommitInput, message: &str) -> Option<LintDiagnostic> {
    let mut index = 0;
    while let Some(at) = message[index..].find('@') {
        let start = index + at;
        let rest = &message[start..];
        let end =
            rest.char_indices().skip(1).find(|(_, ch)| !is_package_char(*ch)).map(|(offset, _)| offset).unwrap_or(rest.len());
        let token = &rest[..end];
        let token_end = start + end;
        if token.contains('/') && !is_backtick_wrapped(message, start, token_end) {
            return diagnostic(
                ctx,
                RULE_COMMIT_BARE_PACKAGE,
                format!("wrap scoped package identifier `{token}` in backticks"),
                commit,
            );
        }
        index = start + end.max(1);
    }
    None
}

fn check_bare_symbols(ctx: &LintContext, commit: &CommitInput, message: &str) -> Option<LintDiagnostic> {
    for token in BARE_SYMBOLS {
        if find_bare_token(message, token).is_some() {
            return diagnostic(ctx, RULE_COMMIT_BARE_SYMBOL, format!("wrap identifier `{token}` in backticks"), commit);
        }
    }
    None
}

fn find_semver(text: &str) -> Option<&str> {
    for (index, _) in text.char_indices() {
        if let Some(token) = semver_at(text, index) {
            return Some(token);
        }
    }
    None
}

fn semver_at(text: &str, start: usize) -> Option<&str> {
    let rest = text.get(start..)?;
    let (raw, prefix_len) = if let Some(stripped) = rest.strip_prefix('v') {
        (stripped, 1)
    }
    else if let Some(stripped) = rest.strip_prefix('V') {
        (stripped, 1)
    }
    else {
        (rest, 0)
    };
    let mut parts = 0usize;
    let mut pos = 0usize;
    while parts < 3 {
        let slice = raw.get(pos..)?;
        let digit_len = slice.chars().take_while(|ch| ch.is_ascii_digit()).map(|ch| ch.len_utf8()).sum::<usize>();
        if digit_len == 0 {
            return None;
        }
        pos += digit_len;
        parts += 1;
        if parts == 3 {
            break;
        }
        if !raw.get(pos..)?.starts_with('.') {
            return None;
        }
        pos += 1;
    }
    if start > 0 {
        let before = text.as_bytes()[start - 1];
        if before.is_ascii_alphanumeric() || before == b'_' {
            return None;
        }
    }
    let end = start + prefix_len + pos;
    if let Some(after) = text.as_bytes().get(end) {
        if after.is_ascii_alphanumeric() || *after == b'_' {
            return None;
        }
    }
    Some(text.get(start..end)?)
}

fn is_package_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '/' | '@' | '.')
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'
}

fn is_backtick_wrapped(text: &str, start: usize, end: usize) -> bool {
    if start >= end {
        return false;
    }
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < text.len() {
        if bytes[index] != b'`' {
            index += 1;
            continue;
        }
        let open = index;
        index += 1;
        while index < text.len() && bytes[index] != b'`' {
            index += 1;
        }
        if index >= text.len() {
            return false;
        }
        let close = index;
        if open < start && end <= close {
            return true;
        }
        index += 1;
    }
    false
}

fn find_bare_token(text: &str, token: &str) -> Option<usize> {
    if token.is_empty() {
        return None;
    }
    let mut index = 0;
    while let Some(found) = text[index..].find(token) {
        let start = index + found;
        let end = start + token.len();
        let before = text[..start].chars().last();
        let after = text[end..].chars().next();
        let left_ok = before.is_none_or(|ch| !is_ident_char(ch) && ch != '`');
        let right_ok = after.is_none_or(|ch| !is_ident_char(ch) && ch != '`');
        if left_ok && right_ok && !is_backtick_wrapped(text, start, end) {
            return Some(start);
        }
        index = end;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{find_bare_token, find_semver, is_backtick_wrapped, lint_commit_hygiene};
    use crate::{
        context::{CommitInput, LintContext},
        rule::{RULE_COMMIT_BARE_PACKAGE, default_commit_rules},
    };

    #[test]
    fn flags_bare_octokit() {
        let ctx = LintContext::new(vec![], default_commit_rules());
        let commit = CommitInput {
            hash: Some("abc".to_string()),
            subject: "🔧 Monitor GitHub Actions via Octokit".to_string(),
            message: Some("🔧 Monitor GitHub Actions via Octokit".to_string()),
        };
        let diagnostics = lint_commit_hygiene(&ctx, &commit);
        assert!(diagnostics.iter().any(|item| item.rule == "commit/bare-symbol"));
    }

    #[test]
    fn accepts_backtick_wrapped_symbol() {
        assert!(is_backtick_wrapped("via `Octokit`", 5, 12));
        assert!(find_bare_token("via `Octokit`", "Octokit").is_none());
    }

    #[test]
    fn flags_semver_in_subject() {
        assert_eq!(find_semver("Close v0.1.18 gate"), Some("v0.1.18"));
    }

    #[test]
    fn accepts_backtick_wrapped_scoped_package() {
        let ctx = LintContext::new(vec![], default_commit_rules());
        let commit = CommitInput {
            hash: Some("abc".to_string()),
            subject: "✨ Expand `@doki-land/nifty` meta package under `projects/packages`".to_string(),
            message: Some("✨ Expand `@doki-land/nifty` meta package under `projects/packages`".to_string()),
        };
        let diagnostics = lint_commit_hygiene(&ctx, &commit);
        assert!(
            !diagnostics.iter().any(|item| item.rule == RULE_COMMIT_BARE_PACKAGE),
            "expected wrapped scoped package to pass: {diagnostics:?}"
        );
    }

    #[test]
    fn flags_bare_scoped_package() {
        let ctx = LintContext::new(vec![], default_commit_rules());
        let commit = CommitInput {
            hash: Some("abc".to_string()),
            subject: "✨ Adopt @vmz/commander for nifty CLI".to_string(),
            message: Some("✨ Adopt @vmz/commander for nifty CLI".to_string()),
        };
        let diagnostics = lint_commit_hygiene(&ctx, &commit);
        assert!(diagnostics.iter().any(|item| item.rule == RULE_COMMIT_BARE_PACKAGE));
    }

    #[test]
    fn accepts_backtick_wrapped_scoped_package_in_body() {
        let ctx = LintContext::new(vec![], default_commit_rules());
        let commit = CommitInput {
            hash: Some("abc".to_string()),
            subject: "🔧 Point root scripts at `nifty publish` and `nifty trust`".to_string(),
            message: Some(
                "🔧 Point root scripts at `nifty publish` and `nifty trust`\n\nAdd root devDependency on `@doki-land/nifty`."
                    .to_string(),
            ),
        };
        let diagnostics = lint_commit_hygiene(&ctx, &commit);
        assert!(
            !diagnostics.iter().any(|item| item.rule == RULE_COMMIT_BARE_PACKAGE),
            "expected wrapped scoped package in body to pass: {diagnostics:?}"
        );
    }

    #[test]
    fn backtick_wrap_uses_absolute_byte_end() {
        let message = "publish `@doki-land/nifty-config` next";
        let start = message.find('@').expect("@");
        let rest = &message[start..];
        let end = rest
            .char_indices()
            .skip(1)
            .find(|(_, ch)| !ch.is_ascii_alphanumeric() && !matches!(ch, '-' | '/' | '@' | '.'))
            .map(|(offset, _)| offset)
            .unwrap_or(rest.len());
        assert!(is_backtick_wrapped(message, start, start + end));
    }

    #[test]
    fn accepts_backtick_wrapped_scoped_package_with_other_backticks() {
        let message = "✨ Add `nifty-config` layout detection and publish `@doki-land/nifty-config`";
        let start = message.rfind('@').expect("@");
        let rest = &message[start..];
        let end =
            rest.char_indices().skip(1).find(|(_, ch)| !is_package_char(*ch)).map(|(offset, _)| offset).unwrap_or(rest.len());
        assert!(is_backtick_wrapped(message, start, start + end));
    }

    fn is_package_char(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || matches!(ch, '-' | '/' | '@' | '.')
    }
}
