# AGENTS

This repository ships [**Nifty**](https://www.npmjs.com/package/@doki-land/nifty) and its Agent Skills as npm packages.

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

```bash
pnpm install
pnpm run build:napi
pnpm exec nifty --help
```

Hybrid monorepo layout: `projects/crates/` (Rust), `projects/packages/` (npm).

**Release (local or after CI):**

```bash
nifty publish          # skips versions already on npm
nifty trust            # required after first publish of a new package name
```

CI also publishes on tag `vX.Y.Z` via `.github/workflows/publish-npm.yml`. Run `nifty trust` locally when a new
`@doki-land/*` package appears.

## Commit messages

Gitmoji-first, English subject and body, identifiers in backticks. See `nifty-commit` skill for lint rules.
