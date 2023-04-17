---
name: nifty-release
description: >-
  Nifty release workflow: bump, publish, trust, upload, change-logs, and install-native.
  Load when cutting versions, npm OIDC publish, Trusted Publisher, or GitHub Release assets.
---

# Nifty release

## Configuration

Create or extend `nifty.config.ts`:

```ts
export default {
    githubToken: process.env.GITHUB_TOKEN,
    authorMap: "documentation/maintenance/author-github.json",
    changelog: {
        releasesDir: "documentation/maintenance/releases",
        repo: "owner/name",
    },
    publish: {
        packages: ["@scope/app", "@scope/app-native"],
    },
};
```

`publish.packages` lists every package name for **`nifty trust`**, including registry-only native sidecars.

## Version bump

```bash
nifty bump              # patch
nifty bump minor
nifty bump --version 0.0.3
nifty bump --dry-run
```

Aligns all `projects/crates/*/Cargo.toml` and `projects/packages/*/package.json` versions.

## npm publish and trust

**Default: publish the whole workspace.** Nifty walks packages in dependency order and **skips versions already on the
registry** (cache + `npm view`). After a release you can run plain `nifty publish` again — only new or bumped packages
are uploaded.

```bash
nifty publish --dry-run
nifty publish --access public
nifty trust --dry-run
nifty trust
```

Optional subset (new package only, or retry one failure):

```bash
nifty publish --package @doki-land/nifty-skills
nifty trust --only @doki-land/nifty-skills
```

**First publish of a new package name** (e.g. `@doki-land/nifty-skills`): run **`nifty trust`** after `nifty publish` so
GitHub Actions OIDC can publish on the next tag. `nifty trust` uses `publish.packages` from `nifty.config.ts` when set,
otherwise all non-private workspace packages.

Local auth: `NPM_TOKEN`, `--npm-token`, `--otp`, or `--totp-secret` (see `nifty publish --help`). Trust live
configuration requires OTP/TOTP.

**npm-tools CI**: tag `vX.Y.Z` triggers `.github/workflows/publish-npm.yml` → `scripts/ci/publish-npm.mjs` (OIDC, no
long-lived token in YAML). Native `.node` files ship as `@doki-land/nifty-*` optional packages, not GitHub Release
assets.

## Reference changelogs

Output is a **draft index** for hand-editing release notes — not the published GitHub Release body.

```bash
nifty change-logs --version 0.0.3
nifty change-logs --from v0.0.2 --to v0.0.3
nifty change-logs --version 0.0.3 --write
nifty change-logs --tags
nifty change-logs lookup --email you@example.com --fetch
```

Default paths: `documentation/maintenance/author-github.json`, `documentation/maintenance/releases/vX.Y.Z.reference.md`.

## GitHub upload

```bash
nifty upload --release --dir dist --tag v0.0.3 --token "$GITHUB_TOKEN"
nifty upload --pages --dir dist
nifty upload --both --dir dist --github-action
```

Platform `.node` binaries are **not** uploaded here — they publish via `nifty publish` / CI matrix.

## CI native staging (maintainers)

```bash
nifty install-native --from-artifacts path/to/native-artifacts
```

Then `nifty publish` includes staged `@doki-land/nifty-<platform>` packages.

## Release checklist

1. `nifty check` on commits since last tag
2. `nifty bump` (or explicit version)
3. Commit version bump, tag `vX.Y.Z`, push branch + tag
4. `nifty publish` (skips packages already at that version on npm) — or let CI publish on tag
5. **`nifty trust`** for any **new** package name not yet on the Trusted Publisher list
6. `nifty change-logs --version X.Y.Z --write` for reference draft
7. `nifty upload --release …` when generic assets need GitHub Release upload
