# nifty-formatter

Workspace formatter and release-note helpers for the [Nifty](https://www.npmjs.com/package/@doki-land/nifty) stack. Depend on `@doki-land/nifty` unless you need this crate directly.

- **`nifty format`** — `cargo fmt` for Rust; Oak CST formatting for TypeScript / JavaScript
- **Release bullets** — gitmoji subjects, author mentions, grouped changelog sections

## Example

```rust
use nifty_formatter::{run_format, RunFormatOptions};

let report = run_format(RunFormatOptions {
    check: true,
    cwd: None,
})?;
```

[docs.rs](https://docs.rs/nifty-formatter) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-formatter)
