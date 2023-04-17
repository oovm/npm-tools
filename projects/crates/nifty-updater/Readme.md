# nifty-updater

Update **Cargo** and **npm/pnpm** dependencies for Nifty hybrid projects.

Registry versions are queried directly (`crates.io` + `registry.npmjs.org`). Nifty patches manifests and refreshes lockfiles with built-in `cargo update` / `pnpm|npm install` only.

## Behavior

| Layout   | Cargo                                              | JavaScript                                                       |
|----------|----------------------------------------------------|------------------------------------------------------------------|
| `cargo`  | `cargo metadata` + crates.io → patch `Cargo.toml` + `cargo update -p` | —                                                                |
| `npm`    | —                                                  | read `package.json` + npm registry → patch + `npm install`       |
| `hybrid` | both                                               | pnpm workspace discovery → patch + `pnpm install` at workspace root |

Interactive mode (`-i`) uses `dialoguer` multi-select.

## CLI

```bash
nifty update
nifty update -i
nifty update -C path/to/repo
```

## Library

```rust
use nifty_updater::{run_update, UpdateOptions};

run_update(UpdateOptions {
    interactive: false,
    cwd: None,
})?;
```

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-updater)
- [docs.rs](https://docs.rs/nifty-updater)

## License

MPL-2.0
