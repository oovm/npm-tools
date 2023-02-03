# @doki-land/nifty

**Gitmoji is the default Nifty convention.** Commit subjects start with a gitmoji and a space; release notes group commits by gitmoji section.

**`nifty-git`** reads repositories through **gix** (pure Rust, no `git` subprocess) — tags, ranges, and commit logs with gitmoji fields attached.

| Crate | Role |
| --- | --- |
| `nifty-types` | Gitmoji types, parsing, validation |
| `nifty-formatter` | Subject/release formatting |
| `nifty-linter` | Rule-based `nifty lint` / `nifty check` |
| `nifty-updater` | `nifty update` dependency upgrades (`-i` interactive) |
| `nifty-git` | gix repository reads |
| `nifty-github` | GitHub REST user lookup (`gh` parity) |
| `nifty-wasi` | WASI Preview 2 component |
| `src/` | Human-friendly TypeScript (`Gitmoji`, `Git`, `Github`, `Nifty`) |

## Configuration

Create `nifty.config.ts` (or `nifty.config.js`) in your project root:

```ts
import { defineConfig } from "@doki-land/nifty";

export default defineConfig(({ layout }) => ({
  githubToken: process.env.GITHUB_TOKEN,
  // Hybrid cargo + npm monorepo roots are auto-detected:
  repoRoot: layout.root,
  cargoRoot: layout.cargoWorkspaceRoot,
  npmRoot: layout.packageManifest?.replace(/[/\\]package\.json$/, ""),
  authorMap: {
    "you@example.com": { login: "octocat" },
  },
}));
```

`createNifty()` loads this file automatically and detects cargo/npm layout from `cwd` (walks upward for `Cargo.toml`, `package.json`, git root, and workspace roots).

Override inline:

```ts
const nifty = await createNifty({ githubToken: "ghp_..." });
console.log(nifty.layout.kind); // "hybrid" in this repo
```

## Update dependencies

```bash
nifty update
nifty update -i
```

Or from TypeScript:

```ts
import { update } from "@doki-land/nifty";

await update({ interactive: true });
```

## Lint commits

```bash
nifty lint
nifty lint --subject "✨ Add feature"
nifty check
```

```ts
import { lint, lintSubjects } from "@doki-land/nifty";

const report = lintSubjects(["✨ Add feature", "Add bad subject"]);
console.log(report.errorCount);
```

## Build

```bash
rustup target add wasm32-wasip2
cargo install cargo-component
cd projects/packages/nifty && npm install && npm run build
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
  crates/
    nifty-types/      # gitmoji types + parsing (Rust)
    nifty-formatter/  # subject/release formatting (Rust)
    nifty-git/        # gix tag/log/range (Rust)
    nifty-github/     # GitHub REST lookup (Rust)
    nifty-wasi/       # wit component
  packages/
    nifty-config/   # defineConfig + loadConfig
    nifty/
      lib/            # jco transpile output (generated)
      src/            # TypeScript bindings
```
