# npm-tools

Monorepo for [**Nifty**](https://www.npmjs.com/package/@doki-land/nifty) — gitmoji-first commit conventions, hybrid
Cargo + npm workspace tooling, and gix-backed git helpers.

## Packages

| npm                                                                                            | Description                        |
|------------------------------------------------------------------------------------------------|------------------------------------|
| [`@doki-land/nifty`](https://www.npmjs.com/package/@doki-land/nifty)                           | CLI + TypeScript API               |
| [`@doki-land/nifty-skills`](https://www.npmjs.com/package/@doki-land/nifty-skills)             | Agent Skills for Nifty workflows   |
| [`@doki-land/nifty-win32-x64`](https://www.npmjs.com/package/@doki-land/nifty-win32-x64)       | Windows x64 native binding         |
| [`@doki-land/nifty-linux-x64`](https://www.npmjs.com/package/@doki-land/nifty-linux-x64)       | Linux x64 native binding           |
| [`@doki-land/nifty-darwin-arm64`](https://www.npmjs.com/package/@doki-land/nifty-darwin-arm64) | macOS Apple Silicon native binding |

Rust crates live under [`projects/crates/`](https://github.com/oovm/npm-tools/tree/dev/projects/crates). The Node-API
surface is built from `nifty-napi` and copied into the platform packages.

## Quick start (consumers)

```bash
npm install @doki-land/nifty
nifty --help
```

## Development

Requires [Rust](https://www.rust-lang.org/) (nightly toolchain as pinned in the repo) and [pnpm](https://pnpm.io/).

```bash
pnpm install
pnpm run build:napi   # build native addon into platform packages
pnpm exec nifty lint
cargo test --release
```

## License

[MPL-2.0](LICENSE)
