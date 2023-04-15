# nifty-uploader

Upload generic release assets to GitHub Release and deploy GitHub Pages.

Platform Node-API binaries (`@doki-land/nifty-*`) are **not** uploaded here. CI builds them in a matrix, runs
`nifty install-native`, and publishes via `nifty publish` (npm Trusted Publisher OIDC).

## CLI

```bash
nifty upload --release --dir dist --tag v0.1.0 --token "$GITHUB_TOKEN"
nifty upload --pages --dir dist
nifty upload --both --dir dist
```

`--github-action` reads `GITHUB_TOKEN`, `GITHUB_REPOSITORY`, `GITHUB_REF_NAME`, and `GITHUB_WORKSPACE`.

### GitHub Actions

```yaml
- run: npx @doki-land/nifty upload --release --dir dist --github-action
```

In the [npm-tools](https://github.com/oovm/npm-tools) repo, CI uses [
`scripts/ci/nifty.mjs`](https://github.com/oovm/npm-tools/blob/dev/scripts/ci/nifty.mjs) to pin or bootstrap the CLI.

Release notes are auto-generated from git history when `--generate-notes` is enabled (default).

Pages deploy pushes to the `gh-pages` branch (adds `.nojekyll` automatically).

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-uploader)
- [docs.rs](https://docs.rs/nifty-uploader)

## License

MPL-2.0
