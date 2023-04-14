# `@doki-land/nifty-skills`

[Agent Skills](https://agentskills.io/specification) for [**Nifty**](https://www.npmjs.com/package/@doki-land/nifty) —
gitmoji commit conventions, hybrid Cargo + npm release workflows, and gix-backed git history tools.

This package is **docs-only**. It teaches agents how to invoke the published `nifty` CLI. It does not replace
`@doki-land/nifty` or its native bindings.

## Install

Install one skill into a project (requires Node.js 18+):

```bash
npx skills add @doki-land/nifty-skills --skill nifty -y
```

List available skills or install globally:

```bash
npx skills add @doki-land/nifty-skills --list
npx skills add @doki-land/nifty-skills --skill nifty-release -y -g
```

## Skills

| Skill               | Load when                                                                  |
|---------------------|----------------------------------------------------------------------------|
| `nifty`             | User mentions Nifty tooling, hybrid monorepos, or you need the command map |
| `nifty-commit`      | Gitmoji commit lint, subject format, `nifty lint` / `nifty check`          |
| `nifty-release`     | `nifty bump`, `publish`, `trust`, `upload`, `change-logs`, CI release      |
| `nifty-git-history` | `nifty commit`, `nifty retime`, object-layer history apply                 |

## Prerequisites

Consumers need the CLI:

```bash
npm install @doki-land/nifty
nifty --help
```

Monorepo authors can use `pnpm exec nifty` after `pnpm install` in [npm-tools](https://github.com/oovm/npm-tools).

## Links

- [npm package](https://www.npmjs.com/package/@doki-land/nifty-skills)
- [Nifty CLI](https://www.npmjs.com/package/@doki-land/nifty)
- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/packages/nifty-skills)

## License

[MPL-2.0](https://www.npmjs.com/package/@doki-land/nifty-skills?activeTab=code)
