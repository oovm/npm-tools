# nifty-uploader

Upload generic release assets to GitHub Release and deploy GitHub Pages.

Platform Node-API binaries (`@doki-land/nifty-*`) are **not** uploaded here. CI builds them in a matrix, runs `nifty install-native`, and publishes via `nifty publish` (npm Trusted Publisher OIDC). GitHub Actions invoke the published CLI through `node scripts/ci/nifty.mjs` (falls back to workspace `pnpm exec nifty` before the first npm release).

## CLI

```bash
nifty upload --release --dir dist --tag v0.1.0 --token "$GITHUB_TOKEN"
nifty upload --pages --dir dist
nifty upload --both --dir dist
```

`--github-action` reads `GITHUB_TOKEN`, `GITHUB_REPOSITORY`, `GITHUB_REF_NAME`, and `GITHUB_WORKSPACE`.

### GitHub Actions

```yaml
- run: node scripts/ci/nifty.mjs upload --release --dir dist --github-action
```

`scripts/ci/nifty.mjs` runs `@doki-land/nifty` from npm when a version is published (pin with `NIFTY_CLI_VERSION`).

Release notes are auto-generated from git history when `--generate-notes` is enabled (default).

Pages deploy pushes to the `gh-pages` branch (adds `.nojekyll` automatically).
