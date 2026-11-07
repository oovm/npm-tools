# nifty-github

GitHub REST helpers for [Nifty](https://www.npmjs.com/package/@doki-land/nifty): resolve users by login, search by email, and map noreply addresses through `author-github.json`. Pairs with `nifty-types` for author metadata.

Depend on `@doki-land/nifty` unless you need this crate directly.

## Example

```rust
use nifty_github::{lookup_user_by_email, user_by_login};

let user = user_by_login("octocat", None)?;
let mapped = lookup_user_by_email(
    "12345678+octocat@users.noreply.github.com",
    r#"{"12345678+octocat@users.noreply.github.com":{"login":"octocat"}}"#,
    None,
    true,
)?;
```

Exposed to TypeScript as `Github` via `@doki-land/nifty`.

[docs.rs](https://docs.rs/nifty-github) · [Nifty readme](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-github)
