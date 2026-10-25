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
    trust: {
        npm: {
            repo: "owner/name",
            file: "publish-npm.yml",
            environment: "NPM_PUBLISH",
        },
    },
};
```

`trust.npm` pins the GitHub Actions Trusted Publisher target for **`nifty trust`** (prefer over `NIFTY_TRUST_*` env vars).
Omit `publish.packages` unless you need a **narrow subset** — default is every workspace package that is not `private: true`.

## Version bump

```bash
nifty bump              # patch
nifty bump minor
nifty bump --version 0.0.3
nifty bump --dry-run
```

Aligns all `projects/crates/*/Cargo.toml` and `projects/packages/*/package.json` versions.

## Repository policy (read first)

In `npm-tools` and other VMZ-family repositories, real versions publish **only** through tag-triggered CI OIDC
(`.github/workflows/publish-npm.yml`). Local `nifty publish` is limited to `--dry-run` and `--placeholder` (never-published
`0.0.0` stubs). Local token/OTP auth is for `nifty trust` setup and for consumer projects that have no OIDC workflow, not
for releasing these repositories. If the tag or CI run fails, stop and report instead of publishing locally.

## npm publish and trust

**Default for consumer workspaces: publish the whole workspace.** Nifty walks packages in dependency order and **skips versions already on the
registry** (cache + `npm view`). After a release you can run plain `nifty publish` again — only new or bumped packages
are uploaded. Inside `npm-tools` itself, see the repository policy above.

```bash
nifty publish --dry-run
nifty publish --access public
nifty publish --placeholder
nifty trust --dry-run
nifty trust
```

**`--placeholder`**: for each publishable workspace package with **no version on the registry**, publish
workspace contents as **`0.0.0`** (claim the name / prep Trusted Publisher). Packages that already exist on
npm are skipped. Publishable means `private` is **not** set (or `false`) in `package.json`; dependents with a
required dependency on a private workspace package are skipped too. Remove `private` from packages you intend to
ship — do not use `publish.packages` to override `private`. Optional `publish.packages` / `--package` only
**narrows** the non-private set. Prefer `nifty publish --placeholder` over per-repo placeholder scripts. Then run
**`nifty trust`**.

Optional subset (new package only, or retry one failure):

```bash
nifty publish --package @doki-land/nifty-skills
nifty trust --only @doki-land/nifty-skills
```

**First publish of a new package name** (e.g. `@doki-land/nifty-skills`): run **`nifty trust`** after `nifty publish` so
GitHub Actions OIDC can publish on the next tag. `nifty trust` uses `publish.packages` from `nifty.config.ts` when set
(subset only — still respects `private`), otherwise all publishable workspace packages.

Local auth: `NPM_TOKEN`, `--npm-token`, `--otp`, or `--totp-secret` (see `nifty publish --help`). Trust uses the
registry API with bearer token from `NPM_TOKEN` or `~/.npmrc`. Add OTP/TOTP on 2FA accounts when the registry asks.
Per the repository policy above, these paths are not used for real releases in `npm-tools` or VMZ-family repos.

**npm-tools CI**: tag `vX.Y.Z` triggers `.github/workflows/publish-npm.yml`. The workflow stages native artifacts into
the platform packages and then runs `nifty publish` (OIDC, no long-lived token in YAML). Native `.node` files ship as
`@doki-land/nifty-*` optional packages, not GitHub Release assets.

## Reference changelogs

Output is a **draft index** for hand-editing release notes — not the published GitHub Release body.

```bash
nifty change-logs --version 0.1.10
nifty change-logs --from v0.1.9 --to v0.1.10
nifty change-logs --version 0.1.10 --write
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
4. Push the tag and let CI publish (OIDC). Do not run a real-version `nifty publish` locally
5. **`nifty trust`** for any **new** package name not yet on the Trusted Publisher list
6. `nifty change-logs --version X.Y.Z --write` for reference draft
7. `nifty upload --release …` when generic assets need GitHub Release upload
