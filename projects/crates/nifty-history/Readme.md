# nifty-history

Commit history rewrite and release reference changelog generation
for [Nifty](https://www.npmjs.com/package/@doki-land/nifty).

| Module      | CLI equivalent                 | Purpose                                                 |
|-------------|--------------------------------|---------------------------------------------------------|
| `commit`    | `nifty commit`, `nifty retime` | Object-layer message apply and timestamp spreading      |
| `changelog` | `nifty change-logs`            | Tag-range reference changelogs and GitHub author lookup |

Built on **[gix](https://github.com/GitoxideLabs/gitoxide)** for commit apply/retime (no interactive rebase). Changelog
collection uses the `git` CLI for tag ranges (same contract as legacy `change-logs` scripts).

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-history)
- [docs.rs](https://docs.rs/nifty-history)

## License

MPL-2.0
