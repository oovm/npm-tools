# AGENTS

This repository ships [**Nifty**](https://www.npmjs.com/package/@doki-land/nifty) and its Agent Skills as npm packages.

## Read order

1. This file: repository layout, build, release policy.
2. `projects/packages/nifty-skills/skills/nifty/SKILL.md`: command map and boundaries.
3. Only the one task skill you need (`nifty-commit`, `nifty-release`, `nifty-git-history`).
4. Public `readme.md` and crate Readmes are product documentation. Read them only when changing published API or
   description, never as design truth for implementation.

`documentation/maintenance/releases/` holds per-release notes; it is history, not current design.

## Agent Skills

Reusable workflows live in [Agent Skills](https://agentskills.io/specification) format under:

```text
projects/packages/nifty-skills/skills/<name>/SKILL.md
```

Published as [`@doki-land/nifty-skills`](https://www.npmjs.com/package/@doki-land/nifty-skills). Install into a consumer
project:

```bash
npx skills add @doki-land/nifty-skills --skill nifty -y
```

| Skill               | When to load                                                |
|---------------------|-------------------------------------------------------------|
| `nifty`             | Nifty CLI overview and command map                          |
| `nifty-commit`      | Gitmoji lint / `nifty check`                                |
| `nifty-release`     | bump, publish, trust, upload, change-logs                   |
| `nifty-git-history` | commit export/apply/scan/audit, retime (object-layer apply) |

When working **inside npm-tools**, read skills from `projects/packages/nifty-skills/skills/` directly. Do not duplicate
skill bodies elsewhere.

## Nifty CLI

Nifty requires Node.js >=24.12.0. The CLI loads built JS from `dist`, so build the package before using the local bin.

```bash
pnpm install
pnpm run build:napi
pnpm --filter @doki-land/nifty run build
pnpm exec nifty --help
```

`build:napi` runs `cargo metadata --locked` first and refuses to compile when `Cargo.lock` is out of sync, so a local
Oak overlay has to be off for it. `pnpm run build` (plain `cargo build --release`) stays patch-friendly, and
`pnpm run build:ci` reproduces the CI `--locked` flags.

Hybrid monorepo layout: `projects/crates/` (Rust), `projects/packages/` (npm).

## Release policy

Real versions (`X.Y.Z`) are published **only** by `.github/workflows/publish-npm.yml` through npm Trusted Publisher
(OIDC), triggered by pushing tag `vX.Y.Z`. Never run a real-version `nifty publish`, `npm publish`, or OTP/TOTP/token
publishing path locally for this repository or another VMZ-family repository.

```bash
nifty bump --patch      # version commit only
git push origin dev
git tag vX.Y.Z && git push origin vX.Y.Z   # CI publishes, then verify the run
nifty trust             # only for a newly added @doki-land/* package name
```

`nifty publish --dry-run` is allowed for inspection. `nifty publish --placeholder` is only for never-published `0.0.0`
stubs that configure Trusted Publisher, never for a real release. If the tag or CI fails, stop and report. Do not
switch to a local publish to finish the task.

`nifty bump` does not rewrite `Cargo.lock`. Member versions are recorded there, so the bumped tree fails `--locked`
until the lock is refreshed. Refresh it in a clean clone (`cargo metadata --offline`, then check that the diff is only
the member versions) and ship it in the same commit as the manifests.

### Pinned toolchain

`rust-toolchain.toml` and every workflow pin `nightly-2026-11-07` (`rustc 1.101.0-nightly`). Nightly is required, not
cosmetic: the pinned `oak` revision uses `#![feature(new_range_api)]`, so stable cannot build the workspace.

Update the pin deliberately, never as a side effect of another change:

- Who: the maintainer cutting the next release.
- When: at most once per release cycle, and only with a green `check-rust` matrix on the resulting commit.
- Trigger: an `oak` revision bump that needs newer syntax, or a compiler fix we need. A stale pin is harmless, a
  floating `nightly` is not — a bad nightly would break the release chain with no commit to point at.

`npm install -g npm@12.2.2` in `publish-npm.yml` is the version that carried the last successful publish. Trusted
Publishing only requires `>= 11.5.1`, so the pin tracks the newest major already proven in this repository rather than
the floor.

`napi build --cargo-flags=--locked` is not the drift guard: `napi` runs an unlocked `cargo metadata` first, so a stale
`Cargo.lock` would be rewritten before the build starts. `build:napi` therefore checks `cargo metadata --locked` before
anything compiles and exits non-zero on a mismatch. `ci-gate` remains the real pin for the published graph — it requires
`check-rust` green with `--locked` on the same commit — and the pre-check only keeps this build honest.

## Commit messages

Gitmoji-first, English subject and body, identifiers in backticks. See `nifty-commit` skill for lint rules. Release
notes live in GitHub Releases and `documentation/maintenance/releases/` only for the current release series.
