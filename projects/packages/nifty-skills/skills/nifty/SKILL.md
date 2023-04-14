---
name: nifty
description: >-
  Orchestrate Nifty CLI workflows for gitmoji commits, hybrid Cargo + npm release,
  and gix-backed git history tools. Load when the user mentions nifty, @doki-land/nifty,
  gitmoji release tooling, or npm-tools monorepo helpers.
---

# Nifty

[Nifty](https://www.npmjs.com/package/@doki-land/nifty) is gitmoji-first release tooling for hybrid **Cargo + npm**
workspaces. Native git work uses **gix** (Rust). Changelog tag ranges use the `git` CLI.

## Before you act

1. Confirm `@doki-land/nifty` is installed (`nifty --help`) or use `pnpm exec nifty` in the npm-tools repo.
2. Look for `nifty.config.ts` at the project root (`defineConfig`, `authorMap`, `publish.packages`, `changelog.*`).
3. Load a focused sub-skill when the task is narrow (see table below).

## Sub-skills

| Task                                                                 | Skill               |
|----------------------------------------------------------------------|---------------------|
| Commit subject lint, gitmoji rules                                   | `nifty-commit`      |
| Version bump, npm publish/trust, GitHub upload, reference changelogs | `nifty-release`     |
| Object-layer commit apply/retime                                     | `nifty-git-history` |

## Command map

| Command                                      | Purpose                                                        |
|----------------------------------------------|----------------------------------------------------------------|
| `nifty lint` / `nifty check`                 | Workspace gitmoji + cargo hygiene                              |
| `nifty commit scan` / `nifty commit audit`   | Commit-only gitmoji + message hygiene                          |
| `nifty commit export` / `nifty commit apply` | Export/apply commit message maps at the object layer           |
| `nifty bump`                                 | Align crate + package versions                                 |
| `nifty publish`                              | Publish workspace npm packages (OIDC or token + TOTP)          |
| `nifty trust`                                | Configure npm Trusted Publisher                                |
| `nifty update`                               | Bump Cargo and npm dependencies                                |
| `nifty format`                               | Format Rust via `cargo fmt` and JS/TS/JSON via `oxc_formatter` |
| `nifty upload`                               | GitHub Release assets or GitHub Pages                          |
| `nifty install-native`                       | Stage CI `.node` artifacts into platform packages              |
| `nifty retime`                               | Spread timestamps on a new branch                              |
| `nifty change-logs`                          | Draft release reference changelogs from tags                   |

Global flag: `-C, --cwd <dir>`.

## Layout (npm-tools monorepo)

```text
projects/crates/     # Rust libraries + nifty-napi
projects/packages/   # @doki-land/nifty + platform @doki-land/nifty-*
scripts/ci/          # publish-npm.mjs, nifty.mjs
```

Do **not** confuse npm-tools `legion` / Valkyrie with Nifty. Nifty is independent Node-API + TypeScript CLI tooling.

## Agent discipline

- Prefer **`nifty` CLI** over reimplementing git/npm steps in ad-hoc scripts.
- Run **`nifty commit audit`** or **`nifty check --from … --to …`** before claiming commit history is clean.
- For publish/trust, follow **`nifty-release`**: `nifty publish` skips registry versions that already exist. Run **
  `nifty trust`** after the first publish of any new package name.
- Object-layer history (**commit apply** / **retime**) rewrites refs — warn the user and require explicit `--dry-run`
  first when appropriate.
