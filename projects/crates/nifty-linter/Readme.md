# nifty-linter

Rule-based lint/check for Nifty gitmoji commit conventions and cargo workspace hygiene (ported from `cargo cry`).

## Gitmoji rules

| Rule              | Default | Description                                          |
|-------------------|---------|------------------------------------------------------|
| `gitmoji/subject` | error   | Subject must start with a known gitmoji + space      |
| `gitmoji/known`   | error   | Gitmoji must be in the Nifty known list              |
| `gitmoji/body`    | warning | Body after gitmoji must not be empty                 |
| `gitmoji/format`  | warning | Subject should match `format_subject(gitmoji, body)` |

## Commit hygiene rules

| Rule                  | Default | Description                                        |
|-----------------------|---------|----------------------------------------------------|
| `commit/semver`       | error   | Semver must not appear in subject or body          |
| `commit/semicolon`    | error   | Message must not contain `;` or `；`               |
| `commit/bare-package` | error   | Scoped npm names (`@scope/pkg`) must use backticks |
| `commit/bare-symbol`  | warning | Common identifiers should use backticks              |

`commit/bare-package` treats backtick-wrapped scoped names as satisfied even when the subject also contains other backtick-wrapped identifiers.

## Cargo workspace rules (`cargo cry`)

| Rule                      | Default | Description                                                 |
|---------------------------|---------|-------------------------------------------------------------|
| `cargo/readme-case`       | error   | `README.md` must be lowercase `readme.md`                   |
| `cargo/package-section`   | error   | `Cargo.toml` must declare `[package]`                       |
| `cargo/readme-missing`    | error   | Crate root must contain `readme.md`                         |
| `cargo/missing-docs`      | error   | `missing_docs` lint at least `warn`                         |
| `cargo/workspace-inherit` | error   | Member `package.*` fields should use `workspace = true`     |
| `cargo/workspace-dep`     | error   | Shared deps should use `workspace = true`                   |
| `cargo/doc-include-str`   | error   | Long `src/readme.md` must use `#![doc = include_str!(...)]` |
| `cargo/misplaced-test`    | error   | No `#[test]` / `mod tests` under `src/`                     |
| `cargo/misplaced-root-rs` | error   | No loose `.rs` files beside `Cargo.toml`                    |
| `cargo/large-file`        | warning | Rust sources over 1000 lines                                |

## Workspace rules

| Rule                      | Default | Description                                                 |
|---------------------------|---------|-------------------------------------------------------------|
| `typescript/large-file`   | warning | TypeScript sources (`.ts`/`.tsx`/`.mts`/`.cts`) over 1000 lines |

Cargo and workspace rules run automatically on hybrid/cargo/npm repos when linting commits. Pass `--no-cargo` to skip them. Use
`--subject` alone to lint only gitmoji rules.

## CLI

Via [`@doki-land/nifty`](https://www.npmjs.com/package/@doki-land/nifty):

```bash
nifty lint
nifty lint --subject "✨ Add feature"
nifty lint --from v0.1.0 --to HEAD
nifty lint --no-cargo
nifty check
```

Configure rule severity in `nifty.config.ts`:

```ts
export default {
  lint: {
    rules: [
      { id: "cargo/large-file", severity: "warning", enabled: true },
      { id: "typescript/large-file", severity: "warning", enabled: true },
    ],
  },
};
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

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-linter)
- [docs.rs](https://docs.rs/nifty-linter)

## License

MPL-2.0
