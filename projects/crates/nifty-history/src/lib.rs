#![warn(missing_docs)]

//! Commit history rewrite (reword/retime) and release reference changelogs.
//!
//! Implementation ported from [git-tools](https://github.com/oovm/git-tools).

/// release 参考 changelog 与 GitHub 作者解析。
pub mod changelog;
/// commit 历史遍历与 message 改写。
pub mod commit;
/// 统一错误类型（`gix-error`）。
pub mod error;
/// 仓库发现与通用 OID 工具。
pub mod repo;

pub use error::{Error, Exn, Message, OptionExt, Result, ResultExt, ensure, message, validation, validation_with_input};
