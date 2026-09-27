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
nifty commit audit --from d25daeba^ --to HEAD --errors-only
nifty commit audit --from d25daeba^ --to HEAD --json

# Export a reword map and annotate violations in-place
nifty commit export --base d25daeba --ref dev --path commit.pending.json --with-lint

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

1. Run `nifty commit audit` on the range about to merge or release.
2. For history rewrites, export with lint annotations:
   `nifty commit export --base <exclusive-base> --ref dev --path commit.pending.json --with-lint`
3. Edit only entries with `violations` (or all subjects that need changes), then apply with `nifty commit apply`.
4. Re-run `nifty commit audit` until exit code is 0.

`scan` and `audit` group findings by commit hash, print a per-rule summary, and support `--json` for automation.
`export --with-lint` writes the same `violations` array onto each JSON entry. `commit apply` ignores that field.

Do not bypass lint with custom regex or one-off scripts when `nifty commit audit` is available.
