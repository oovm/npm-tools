---
name: nifty-commit
description: >-
  Lint and enforce Nifty gitmoji commit conventions with nifty lint, nifty check, and nifty commit scan/audit.
  Load when fixing commit subjects, gitmoji format, or pre-release commit hygiene.
---

# Nifty commit conventions

## Subject shape

Every commit **subject** (first line) must:

1. Start with a **known gitmoji** immediately followed by a **space**
2. Use an **English** imperative phrase after the gitmoji (project convention for publish repos)
3. Match `gitmoji + body` formatting rules enforced by `nifty lint`

Known gitmojis (Nifty default set):

```text
✨ 🎨 🚀 🐛 🚑 🔥 💥 ♻️ 🔧 📝 👷 🧹 ⬆️ 🧪 🔨 📦
```

## Commands

```bash
# Workspace lint (commits + optional cargo hygiene)
nifty lint --from v0.0.1 --to HEAD
nifty check --from origin/dev..HEAD

# Commit-only scan/audit (gitmoji + semver/semicolon/backtick hygiene)
nifty commit scan --from d25daeba^ --to HEAD
nifty commit audit --from d25daeba^ --to HEAD

# Lint explicit strings
nifty lint --subject "✨ Add feature"
```

## Rules (nifty-linter)

| Rule                  | Default | Meaning                                   |
|-----------------------|---------|-------------------------------------------|
| `gitmoji/subject`     | error   | Subject must start with gitmoji + space   |
| `gitmoji/known`       | error   | Gitmoji must be in the known list         |
| `gitmoji/body`        | warning | Text after gitmoji must not be empty      |
| `gitmoji/format`      | warning | Subject should match canonical formatting |
| `commit/semver`       | error   | Semver must not appear in subject/body    |
| `commit/semicolon`    | error   | Message must not contain `;` or `；`      |
| `commit/bare-package` | error   | Scoped npm names must use backticks       |
| `commit/bare-symbol`  | warning | Common identifiers should use backticks   |

## Commit message style (when you write commits)

Align with Nifty/npm-tools git policy when the user asks you to commit:

- Subject starts with one real gitmoji character
- Subject and body in **English**
- No Conventional Commit prefix (`feat:`, `fix:`) instead of gitmoji
- Wrap identifiers in backticks: `` `nifty-skills` ``, `` `publish-npm.mjs` ``
- No semicolons in commit messages
- One gitmoji at the start of the subject only

## Workflow

1. Run `nifty commit scan` on the range about to merge or release.
2. Fix subjects locally or plan a **`nifty commit export` / `nifty commit apply`** pass (see `nifty-git-history`) for
   history already pushed.
3. Re-run `nifty commit audit` until exit code is 0.

Do not bypass lint with custom regex when `nifty commit scan` is available.
