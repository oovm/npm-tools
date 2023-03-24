# nifty-github

GitHub REST API helpers for [Nifty](https://www.npmjs.com/package/@doki-land/nifty).

| API | Token | Notes |
| --- | --- | --- |
| `user_by_login` | optional | `GET /users/{login}` |
| `search_user_by_email` | required | `GET /search/users?q=… in:email` |
| `lookup_user_by_email` | optional | noreply → map → search → optional fetch |

Pairs with [`nifty-types`](https://docs.rs/nifty-types) for noreply emails and `author-github.json` maps.

```rust
use nifty_github::{lookup_user_by_email, user_by_login};

let user = user_by_login("oovm", None)?;
let mapped = lookup_user_by_email(
    "aster@vers.site",
    r#"{"aster@vers.site":{"login":"oovm"}}"#,
    None,
    true,
)?;
```

## Links

- [Source](https://github.com/oovm/npm-tools/tree/dev/projects/crates/nifty-github)
- [docs.rs](https://docs.rs/nifty-github)

## License

MPL-2.0
