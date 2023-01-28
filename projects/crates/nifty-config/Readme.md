# nifty-config

Discover Nifty project configuration (`nifty.config.ts` / `nifty.config.js`) by walking upward from a directory.

On the npm side, use `@doki-land/nifty-config` for `defineConfig`, `loadConfig`, and cargo/npm project detection.

```rust,no_run
fn main() -> std::io::Result<()> {
    use std::env::current_dir;
    use nifty_config::{detect_project_layout, find_config_file};

    let _layout = detect_project_layout(&current_dir()?);
    let _path = find_config_file(&current_dir()?);
    Ok(())
}
```
