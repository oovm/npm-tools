# nifty-git

Read-only git helpers for [Nifty](https://www.npmjs.com/package/@doki-land/nifty): discover repo roots, list semver tags, walk commit history, resolve release ranges, and parse gitmoji subjects. Built on **[gix](https://github.com/GitoxideLabs/gitoxide)** — no `git` subprocess for these reads.

Depend on `@doki-land/nifty` unless you need this crate directly.

## Example

```rust
use nifty_git::{collect_commits, discover_root, resolve_range};

let root = discover_root(".")?;
let range = resolve_range(&root, Some("0.1.0"), None, None)?;
let commits = collect_commits(&root, range.from_ref.as_deref(), &range.to_ref)?;
```

Exposed to TypeScript as `Git` via `@doki-land/nifty`.

[docs.rs](https://docs.rs/nifty-git) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-git)
