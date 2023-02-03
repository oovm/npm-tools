# nifty-updater

Update dependencies for Nifty hybrid cargo + npm projects.

## CLI

```bash
cargo install --path projects/crates/nifty-updater
nifty update
nifty update -i
```

- `nifty update` — runs `cargo upgrade --workspace` and `npm update` where detected
- `nifty update -i` — interactive multiselect via `cargo upgrade --dry-run` and `npm outdated`

Project layout is detected through `nifty-config` (git root, workspace roots, `projects/packages/*`).
