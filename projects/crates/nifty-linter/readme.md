# nifty-linter

Gitmoji commit lint and hybrid workspace hygiene for [Nifty](https://www.npmjs.com/package/@doki-land/nifty). Covers subject rules, semver bans, Cargo `readme.md` conventions, and TypeScript size warnings. Depend on `@doki-land/nifty` unless you need this crate directly.

## Rules (summary)

| Group | Examples |
| --- | --- |
| Gitmoji | `gitmoji/subject`, `gitmoji/known`, `gitmoji/body`, `gitmoji/format` |
| Commit hygiene | `commit/semver`, `commit/semicolon`, `commit/bare-package` |
| Cargo workspace | `cargo/readme-case`, `cargo/workspace-inherit`, `cargo/misplaced-test` |
| TypeScript | `typescript/large-file` |

Configure severity in `nifty.config.ts` under `lint.rules`. Pass `--no-cargo` to skip Cargo checks.

## Example

```rust
use nifty_linter::{run_lint, LintOptions};

let report = run_lint(LintOptions {
    subjects: vec!["✨ Add feature".into()],
    ..Default::default()
})?;
report.print_human();
```

CLI: `nifty lint` · `nifty check`

[docs.rs](https://docs.rs/nifty-linter) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-linter)
