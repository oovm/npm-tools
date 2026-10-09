# nifty-config

Find and interpret Nifty project configuration (`nifty.config.ts` / `.js`) and detect hybrid Cargo + npm layout. TypeScript helpers (`defineConfig`, `loadConfig`) ship from `@doki-land/nifty`.

Depend on `@doki-land/nifty` unless you need this crate directly.

## Example

```rust
use nifty_config::{detect_project_layout, find_config_file};

let layout = detect_project_layout(".")?;
let config_path = find_config_file(".");
```

[docs.rs](https://docs.rs/nifty-config) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-config)
