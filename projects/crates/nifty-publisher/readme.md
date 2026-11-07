# nifty-publisher

Topological npm workspace publish for Nifty hybrid monorepos — discovers `projects/packages/*`, orders by internal `file:` / `workspace:` links, patches dependency specs to semver for publish, then restores manifests.

Depend on `@doki-land/nifty` unless you need this crate directly.

## Example

```rust
use std::path::PathBuf;
use nifty_publisher::{publish_workspace, PublishOptions};

let report = publish_workspace(PublishOptions {
    cwd: Some(PathBuf::from(".")),
    dry_run: true,
    ..Default::default()
})?;
```

CLI: `nifty publish`

[docs.rs](https://docs.rs/nifty-publisher) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-publisher)
