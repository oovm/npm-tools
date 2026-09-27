# nifty-formatter

Nifty formatting helpers and workspace formatter.

## Gitmoji and release notes

- `format_subject` — gitmoji-prefixed commit subjects
- `commit_bullet` / `author_mention` — release reference bullets
- `format_release_section` / `format_release_notes` — grouped changelog markdown

## Workspace format (`nifty format`)

| Target                              | Engine                                    |
|-------------------------------------|-------------------------------------------|
| `*.rs` (Cargo workspace)            | `cargo fmt --all`                         |
| `*.ts` / `*.js` / `*.jsx` / `*.tsx` | `oxc_formatter` (git pin `oxfmt_v0.70.0`) |

Default JS style comes from `nifty.config` `format.style` (4 spaces, single quotes, line width 144).

```bash
nifty format
nifty format --check
```

## Library

```rust
use nifty_formatter::{format_source, run_format, RunFormatOptions};

let report = run_format(RunFormatOptions {
    check: false,
    cwd: None,
})?;
```

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-formatter)
- [docs.rs](https://docs.rs/nifty-formatter)

## License

MPL-2.0
