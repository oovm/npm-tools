//! Git 仓库发现与引用解析（gix）。

use std::path::{Path, PathBuf};

use gix::{ObjectId, Repository};
use nifty_types::{RefDomain, RefError};

pub type Result<T> = std::result::Result<T, String>;

/// 向上发现 git 工作区根目录。
pub fn discover_root(start: &Path) -> Result<PathBuf> {
    let repo = gix::discover(start).map_err(|err| err.to_string())?;
    repo.work_dir().map(Path::to_path_buf).ok_or_else(|| "bare repository has no working directory".to_string())
}

/// 打开仓库（从路径向上 discover）。
pub fn open_repo(start: &Path) -> Result<Repository> {
    gix::discover(start).map_err(|err| err.to_string())
}

/// 将 revision 解析为 commit OID，annotated tag 会被剥离到它指向的 commit。
pub fn resolve_rev(repo: &Repository, reference: &str) -> Result<ObjectId> {
    resolve_rev_for(repo, reference, RefDomain::Ref)
}

/// 同 [`resolve_rev`]，但保留调用方的领域标签用于错误文案。
fn resolve_rev_for(repo: &Repository, reference: &str, domain: RefDomain) -> Result<ObjectId> {
    let id = repo.rev_parse_single(reference).map_err(|err| rev_parse_error(reference, domain, &err))?;
    let object = id.object().map_err(|err| err.to_string())?;
    let kind = object.kind.to_string();
    let commit = object.peel_to_commit().map_err(|err| peel_error(reference, domain, &kind, &err))?;
    Ok(commit.id)
}

/// gix 的 `rev-parse` 失败归入统一口径；无法分类的部分保留原文。
fn rev_parse_error(reference: &str, domain: RefDomain, err: &gix::revision::spec::parse::single::Error) -> String {
    use gix::revision::spec::parse::{Error as ParseError, single::Error as SingleError};

    let class = match err {
        SingleError::Parse(ParseError::FindReference(gix::refs::file::find::existing::Error::NotFound { .. }))
        | SingleError::Parse(ParseError::PrefixNotFound { .. }) => {
            RefError::NotFound { reference: reference.to_string(), domain }
        }
        SingleError::Parse(ParseError::AmbiguousPrefix { .. } | ParseError::AmbiguousRefAndObject { .. }) => {
            RefError::Ambiguous { reference: reference.to_string(), detail: err.to_string() }
        }
        _ => RefError::Other { detail: err.to_string() },
    };
    class.to_string()
}

/// gix 能读到对象但无法剥离成 commit：对象存在，但不是 commit。
///
/// `kind` 取剥离前的对象类型，因此 annotated tag 报 `tag`，而不是它最终指向的 `tree`。
fn peel_error(reference: &str, domain: RefDomain, kind: &str, err: &gix::object::peel::to_kind::Error) -> String {
    let class = match err {
        gix::object::peel::to_kind::Error::NotFound { .. } => {
            RefError::NotCommit { reference: reference.to_string(), kind: kind.to_string(), domain }
        }
        _ => RefError::Other { detail: err.to_string() },
    };
    class.to_string()
}

/// 校验 ref 可解析为 commit。
pub fn verify_commit_ref(repo: &Repository, reference: &str, domain: RefDomain) -> Result<()> {
    resolve_rev_for(repo, reference, domain).map(|_| ())
}

/// OID 短 hash（8 位）。
pub fn short_oid(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
