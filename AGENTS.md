# AGENTS

This repository ships [**Nifty**](https://www.npmjs.com/package/@doki-land/nifty) and its Agent Skills as npm packages.

## Read order

1. This file: repository layout, build, release policy.
2. `projects/packages/nifty-skills/skills/nifty/SKILL.md`: command map and boundaries.
3. Only the one task skill you need (`nifty-commit`, `nifty-release`, `nifty-git-history`).
4. Public `Readme.md` and crate Readmes are product documentation. Read them only when changing published API or
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

## Commit messages

Gitmoji-first, English subject and body, identifiers in backticks. See `nifty-commit` skill for lint rules. Release
notes live in GitHub Releases and `documentation/maintenance/releases/` only for the current release series.
