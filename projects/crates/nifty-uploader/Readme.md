# nifty-uploader

Upload generic release assets to GitHub Release and deploy GitHub Pages.

Platform Node-API binaries (`@doki-land/nifty-*`) are **not** uploaded here. CI builds them in a matrix, merges artifacts into `projects/packages/nifty-*/lib/`, and publishes via `nifty publish` (npm Trusted Publisher OIDC), same pattern as vmz-framework.

## CLI

```bash
nifty upload --release --dir dist --tag v0.1.0 --token "$GITHUB_TOKEN"
nifty upload --pages --dir dist
nifty upload --both --dir dist
```

`--github-action` reads `GITHUB_TOKEN`, `GITHUB_REPOSITORY`, `GITHUB_REF_NAME`, and `GITHUB_WORKSPACE`.

Release notes are auto-generated from git history when `--generate-notes` is enabled (default).

Pages deploy pushes to the `gh-pages` branch (adds `.nojekyll` automatically).
