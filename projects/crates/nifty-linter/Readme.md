# nifty-linter

Rule-based lint/check for Nifty gitmoji commit conventions.

## Rules

| Rule | Default | Description |
| --- | --- | --- |
| `gitmoji/subject` | error | Subject must start with a known gitmoji + space |
| `gitmoji/known` | error | Gitmoji must be in the Nifty known list |
| `gitmoji/body` | warning | Body after gitmoji must not be empty |
| `gitmoji/format` | warning | Subject should match `format_subject(gitmoji, body)` |

## CLI

Via npm `@doki-land/nifty` (`cli/nifty.mjs`, Node.js CLI):

```bash
nifty lint
nifty lint --subject "✨ Add feature"
nifty lint --from v0.1.0 --to HEAD
nifty check
```

## Library

```rust
use nifty_linter::{run_lint, LintOptions};

let report = run_lint(LintOptions {
    subjects: vec!["✨ Add feature".to_string()],
    ..Default::default()
})?;
report.print_human();
```
