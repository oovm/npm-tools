# nifty-uploader

Publish artifacts to GitHub Release and GitHub Pages with a single native binary — faster than chaining third-party Actions.

## CLI

```bash
nifty upload --release --dir target/release --tag v0.1.0 --token "$GITHUB_TOKEN"
nifty upload --pages --dir dist
nifty upload --both --dir dist
```

`--github-action` reads `GITHUB_TOKEN`, `GITHUB_REPOSITORY`, `GITHUB_REF_NAME`, and `GITHUB_WORKSPACE` for CI. **本仓在 `0.0.0` 发布前尚未在 GitHub Actions 中启用 `nifty upload`**，当前 release 仍由 `publish.yml` 里的 `upload-release-action` 上传二进制。

### GitHub Actions（`0.0.0` 后可选）

```yaml
- uses: actions/checkout@v4
- uses: hecrj/setup-rust-action@v2
- run: cargo build --release --manifest-path projects/crates/nifty-updater/Cargo.toml
- name: Upload release assets
  run: |
    cargo run --release --manifest-path projects/crates/nifty-updater/Cargo.toml -- \
      upload --release --dir target/release --github-action
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

Release notes are auto-generated from git history via `nifty-formatter` when `--generate-notes` is enabled (default).

Pages deploy pushes to the `gh-pages` branch (adds `.nojekyll` automatically).
