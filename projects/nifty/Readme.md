# @doki-land/nifty

**Gitmoji is the default Nifty convention.** Commit subjects start with a gitmoji and a space; release notes group commits by gitmoji section.

**`nifty-git`** reads repositories through **gix** (pure Rust, no `git` subprocess) — tags, ranges, and commit logs with gitmoji fields attached.

| Crate | Role |
| --- | --- |
| `nifty-core` | Gitmoji spec, authors, bullets |
| `nifty-git` | gix repository reads |
| `nifty-github` | GitHub REST user lookup (`gh` parity) |
| `nifty-wasi` | WASI Preview 2 component |
| `src/` | Human-friendly TypeScript (`Gitmoji`, `Git`, `Github`, `Nifty`) |

## Build

```bash
rustup target add wasm32-wasip2
cargo install cargo-component
cd projects/nifty && npm install && npm run build
```

## Usage

```ts
import { createNifty } from "@doki-land/nifty";

const nifty = await createNifty();

// Gitmoji convention
nifty.gitmoji.validateSubject("✨ Add feature");
nifty.gitmoji.formatSubject("📝", "Update readme");

// Git repository (gix)
const root = nifty.git.discoverRoot(".");
const range = nifty.git.resolveRange(root, { version: "0.1.0" });
const commits = nifty.git.collectCommits(root, range.fromRef, range.toRef);

for (const commit of commits) {
  console.log(nifty.gitmoji.commitBullet(commit.body, commit.email, commit.author));
}

// GitHub API (host fetch; optional token)
const user = await nifty.github.userByLogin("octocat");
const byEmail = await nifty.github.lookupUserByEmail("aster@vers.site", {
  authorMapJson: '{"aster@vers.site":{"login":"oovm"}}',
});
```

## Layout

```text
projects/
  nifty-core/       # gitmoji + author map (Rust)
  nifty-git/        # gix tag/log/range (Rust)
  nifty-github/     # GitHub REST lookup (Rust)
  nifty-wasi/       # wit component
  nifty/
    lib/            # jco transpile output (generated)
    src/            # TypeScript bindings
```
