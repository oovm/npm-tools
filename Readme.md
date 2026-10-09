# Nifty

[![npm @doki-land/nifty](https://img.shields.io/npm/v/@doki-land/nifty.svg)](https://www.npmjs.com/package/@doki-land/nifty)
[![License: MPL-2.0](https://img.shields.io/badge/License-MPL--2.0-blue.svg)](LICENSE)

[Agent Skills](https://www.npmjs.com/package/@doki-land/nifty-skills) · [CLI package](https://www.npmjs.com/package/@doki-land/nifty) · [Issues](https://github.com/oovm/npm-tools/issues)

## 💡 What is Nifty?

**Nifty** is gitmoji-first release tooling for hybrid **Cargo + npm** workspaces. It keeps commit messages, version bumps, formatting, lint, publishing, and changelog drafts aligned across Rust crates and JavaScript packages — without shelling out to `git` for core history work.

Nifty is **not** a general-purpose task runner or a replacement for `cargo` / `npm`. It focuses on conventions, workspace hygiene, and release workflows you repeat on every hybrid monorepo.

## 🚀 Getting started

### 1. Agent / prompt (recommended)

Teach your coding agent the Nifty workflows:

```bash
npx skills add @doki-land/nifty-skills --skill nifty -y
```

Then ask it to run `nifty check`, `nifty bump`, or `nifty publish` against your repo. Load focused skills when the task is narrow:

| Skill | Use when |
| --- | --- |
| `nifty` | Command map and workspace overview |
| `nifty-commit` | Gitmoji lint and commit message rules |
| `nifty-release` | Bump, publish, trust, upload, changelogs |
| `nifty-git-history` | Object-layer commit apply / retime |

### 2. Manual install

```bash
pnpm add -D @doki-land/nifty
nifty --help
```

Add a project config when you need author maps, trust settings, or format presets:

```ts
// nifty.config.ts
import { defineConfig } from "@doki-land/nifty";

export default defineConfig({
  authorMap: "./author-github.json",
});
```

## ✨ Highlights

- **Gitmoji-first commits** — lint subjects, bodies, semver bans, and scoped package naming in one pass (`nifty lint`, `nifty check`).
- **Hybrid versioning** — bump shared `workspace.package` versions and explicit crate / package manifests together (`nifty bump`).
- **Ordered npm publish** — topological workspace publish with dependency patching (`nifty publish`).
- **Oak-backed formatting** — `cargo fmt` for Rust and Oak CST formatting for TypeScript / JavaScript (`nifty format`).
- **gix-powered git reads** — tag ranges, commit walks, and gitmoji parsing without a `git` subprocess for core APIs.
- **Trusted Publisher aware** — `nifty trust` helpers for npm OIDC after first publish of a new package name.

## 📊 How Nifty compares

| Concern | Typical ad-hoc scripts | Nifty |
| --- | --- | --- |
| Commit style | Custom regex in CI | Shared gitmoji rules + `nifty check` |
| Hybrid bump | Manual `Cargo.toml` + `package.json` edits | One `nifty bump --version X.Y.Z` |
| Workspace publish | Manual ordering / forgotten `file:` links | Dependency-ordered `nifty publish` |
| Agent guidance | Undocumented shell one-offs | Published Agent Skills with command map |

## 🧪 Examples

| Goal | Entry point |
| --- | --- |
| Lint the current branch | `nifty check --from v0.1.0 --to HEAD` |
| Format the workspace | `nifty format` · `nifty format --check` |
| Draft release notes | `nifty change-logs --from v0.1.0 --to HEAD` |
| Publish npm packages | `nifty publish` (after `nifty trust` for new names) |

Package-level export maps, native sidecars, and programmatic APIs live in the [@doki-land/nifty](https://www.npmjs.com/package/@doki-land/nifty) readme.

## 📜 License

[MPL-2.0](LICENSE)
