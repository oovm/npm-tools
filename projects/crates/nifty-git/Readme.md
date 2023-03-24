# nifty-git

Read-only git helpers for [Nifty](https://www.npmjs.com/package/@doki-land/nifty): discover repos, list semver tags, walk commit history, resolve release ranges.

Built on **[gix](https://github.com/GitoxideLabs/gitoxide)** — no `git` subprocess. Each commit record includes Nifty **gitmoji** parsing (`body`, `section`, etc.).

## API (Rust)

```rust
use nifty_git::{collect_commits, discover_root, resolve_range};

let root = discover_root(".")?;
let range = resolve_range(&root, Some("0.1.0"), None, None)?;
let commits = collect_commits(&root, range.from_ref.as_deref(), &range.to_ref)?;
```

Exposed to TypeScript via `@doki-land/nifty` (`Git` class) through `nifty-napi`.

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-git)
- [docs.rs](https://docs.rs/nifty-git)

## License

MPL-2.0
