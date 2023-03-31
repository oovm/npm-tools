# nifty-updater

Update **Cargo** and **npm/pnpm** dependencies for Nifty hybrid projects.

## Behavior

| Layout | Cargo | JavaScript |
| --- | --- | --- |
| `cargo` | `cargo upgrade --workspace` | — |
| `npm` | — | `npm outdated` / `npm update` (or per-package in workspaces) |
| `hybrid` | both | `pnpm update -r` when `pnpm-workspace.yaml` is present, else npm |

Interactive mode (`-i`) uses `dialoguer` multi-select for cargo and JS upgrades.

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
