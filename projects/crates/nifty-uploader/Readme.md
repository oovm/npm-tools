# nifty-uploader

Upload release assets to GitHub Releases and deploy static sites to GitHub Pages for Nifty projects. Platform Node-API binaries are published separately via `nifty publish`, not through this uploader.

Depend on `@doki-land/nifty` unless you need this crate directly.

## Example

```bash
nifty upload --release --dir dist --tag v0.1.0
nifty upload --pages --dir dist
```

Use `--github-action` in CI to read `GITHUB_TOKEN`, `GITHUB_REPOSITORY`, and related env vars.

[docs.rs](https://docs.rs/nifty-uploader) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-uploader)
