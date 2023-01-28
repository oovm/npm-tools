# nifty-formatter

Nifty formatting helpers:

- `format_subject` — gitmoji-prefixed commit subjects
- `author_mention` / `commit_bullet` — release note bullets
- `format_release_section` / `format_release_notes` — grouped changelog markdown

Built on [`nifty-types`](../nifty-types) for parsing and section metadata.

```rust
use nifty_formatter::{format_subject, commit_bullet};

let subject = format_subject("✨", "Add feature");
let bullet = commit_bullet("Add feature", "dev@example.com", "Dev", &Default::default());
```
