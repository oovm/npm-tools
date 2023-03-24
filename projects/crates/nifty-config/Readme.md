# nifty-config

Discover Nifty project configuration (`nifty.config.ts` / `nifty.config.js`) by walking upward from a directory.

On the npm side, use [`@doki-land/nifty`](https://www.npmjs.com/package/@doki-land/nifty) for `defineConfig`, `loadConfig`, and Cargo/npm project detection.

```rust,no_run
fn main() -> std::io::Result<()> {
    use std::env::current_dir;
    use nifty_config::{detect_project_layout, find_config_file};

    let _layout = detect_project_layout(&current_dir()?);
    let _path = find_config_file(&current_dir()?);
    Ok(())
}
```

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-config)
- [docs.rs](https://docs.rs/nifty-config)

## License

MPL-2.0
