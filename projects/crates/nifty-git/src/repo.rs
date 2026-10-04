//! Git 仓库发现与引用解析（gix）。

use std::path::{Path, PathBuf};

use gix::{ObjectId, Repository};

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

/// 将 revision 解析为 commit OID。
pub fn resolve_rev(repo: &Repository, reference: &str) -> Result<ObjectId> {
    Ok(repo.rev_parse_single(reference).map_err(|err| err.to_string())?.detach())
}

/// 校验 ref 可解析为 commit。
pub fn verify_commit_ref(repo: &Repository, reference: &str) -> Result<()> {
    resolve_rev(repo, reference).map(|_| ())
}

/// OID 短 hash（8 位）。
pub fn short_oid(oid: ObjectId) -> String {
    oid.to_string().chars().take(8).collect()
}
