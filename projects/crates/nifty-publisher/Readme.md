# nifty-publisher

Publish npm workspace packages in **dependency order** for Nifty hybrid monorepos.

- Discovers packages under `projects/packages/*/package.json`
- Builds an internal dependency graph (`file:` / `workspace:` links)
- Topological sort via [`petgraph`](https://docs.rs/petgraph)
- Patches workspace dependency specs to semver before `npm publish`
- Restores each `package.json` after publish

```rust
use std::path::PathBuf;

use nifty_publisher::{PublishOptions, publish_workspace};

let report = publish_workspace(PublishOptions {
    cwd: Some(PathBuf::from(".")),
    dry_run: true,
    tag: None,
    access: Some("public".to_string()),
})?;
```

Used by `@doki-land/nifty` `nifty publish` through `nifty-napi`.
