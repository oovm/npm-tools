---
name: nifty-git-history
description: >-
  Object-layer git history tools: nifty commit apply/export and nifty retime without interactive rebase.
  Load when rewriting commit messages or timestamps while preserving author identity.
---

# Nifty git history

Implemented in `nifty-history` (gix). **Author and committer name/email are never replaced** with local `user.name` /
`user.email` — only messages or timestamps change (plus parent relinking when ancestors rewrite).

Warn the user before rewriting shared branches. Prefer `--dry-run` first.

## Commit message map

Export JSON map, preview, apply:

```bash
nifty commit export --base 34e1e665^ --ref dev --path commit.pending.json
nifty commit apply --base 34e1e665^ --ref dev --path commit.pending.json --dry-run
nifty commit apply --base 34e1e665^ --ref dev --path commit.pending.json
```

Scan or audit the same commit range before editing:

```bash
nifty commit scan --from 34e1e665^ --to dev
nifty commit audit --from 34e1e665^ --to dev
```

### Map format

Document shape:

```json
{
  "version": 1,
  "entries": [
    { "hash": "abc123…", "message": "✨ Subject\n\nBody." }
  ]
}
```

Flat `{ "abc12345": "message" }` is also accepted.

Hash prefixes must be **unique** within the rewrite range.

## Retime

Spread author and committer timestamps across a window. Writes a **new branch** (default `time-travel`); current branch
unchanged.

Range mode — commits in `(commit..tip]`:

```bash
nifty retime 2a990148
nifty retime 2a990148 --start-date 2019-01-01 --end-date 2019-06-01 --branch dev-time-travel
nifty retime 2a990148 -s 2019-03-22T09:00:00 --tip HEAD
```

Root mode — all commits from repository root through `tip`:

```bash
nifty retime root
nifty retime root -s 2019-01-01 --message "🎂 Project initialized!" --branch time-travel
```

Datetime: `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM:SS`. End defaults to `start + N days` where `N` is commit count in range.

## Change logs (tag ranges)

For release reference drafts, use **`nifty-release`** (`nifty change-logs`). That command uses `git log` for tag ranges;
commit apply/retime use gix object writes.

## Safety

| Do                                         | Do not                                                         |
|--------------------------------------------|----------------------------------------------------------------|
| `--dry-run` before `commit apply`          | Force-push without user consent                                |
| Explicit `--ref` when HEAD is detached     | Use `git rebase -i` when user asked for Nifty tools            |
| Explain parent relink-only rows in dry-run | Amend commits with `git commit --amend` for bulk history fixes |

After apply, the user must force-update the target ref if it was already published.
