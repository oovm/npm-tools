# nifty-updater

Update **Cargo** and **npm / pnpm** dependencies for Nifty hybrid layouts. Queries `crates.io` and the npm registry, patches manifests, and refreshes lockfiles with `cargo update` or `pnpm|npm install`.

Depend on `@doki-land/nifty` unless you need this crate directly.

## Example

```bash
nifty update
nifty update -i
```

```rust
use nifty_updater::{run_update, UpdateOptions};

run_update(UpdateOptions {
    interactive: false,
    cwd: None,
})?;
```

[docs.rs](https://docs.rs/nifty-updater) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-updater)
