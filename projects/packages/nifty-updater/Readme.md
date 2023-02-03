# @doki-land/nifty-updater

Update cargo and npm dependencies for hybrid Nifty projects.

## CLI

```bash
npm install -g @doki-land/nifty-updater
nifty update
nifty update -i
```

Or install the Rust binary directly:

```bash
cargo install --path projects/crates/nifty-updater
```

## Programmatic

```ts
import { update } from "@doki-land/nifty-updater";

await update({ interactive: true });
```

- `nifty update` — `cargo upgrade --workspace` + `npm update`
- `nifty update -i` — interactive multiselect from dry-run / outdated lists

Project layout is auto-detected via `@doki-land/nifty-config`.
