# @doki-land/nifty-skills

[Agent Skills](https://agentskills.io/specification) that teach coding agents how to run [**Nifty**](https://www.npmjs.com/package/@doki-land/nifty) — gitmoji lint, hybrid bumps, publish/trust, formatting, and git history tooling. This package is documentation only; install `@doki-land/nifty` for the CLI and native bindings.

Product overview: [npm-tools readme](https://github.com/oovm/npm-tools#readme).

## Skills

| Skill | Load when |
| --- | --- |
| `nifty` | Command map, workspace layout, when to delegate to sub-skills |
| `nifty-commit` | Gitmoji subject/body rules, `nifty lint`, `nifty check` |
| `nifty-release` | `nifty bump`, `publish`, `trust`, `upload`, `change-logs` |
| `nifty-git-history` | `nifty commit`, `nifty retime`, object-layer history apply |

## Example

Install the overview skill into a consumer project:

```bash
npx skills add @doki-land/nifty-skills --skill nifty -y
```

Example agent prompt after install:

```text
Audit gitmoji commits from v0.1.2 to HEAD with nifty check, then draft a release changelog.
```

Requires `@doki-land/nifty` on the PATH (`nifty --help`).
