//! Cross-ecosystem workspace hygiene (TypeScript sources, etc.).

use std::{fs, path::PathBuf};

use walkdir::WalkDir;

const MAX_TYPESCRIPT_LINES: usize = 1000;
const SKIP_WALK_DIRS: &[&str] = &["node_modules", "target", ".git", "dist", ".cache"];

const TYPESCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "mts", "cts"];

fn should_walk_entry(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }
    let name = entry.file_name().to_string_lossy();
    !name.starts_with(".cry-") && !SKIP_WALK_DIRS.contains(&name.as_ref())
}

fn is_typescript_source(path: &std::path::Path) -> bool {
    let ext = path.extension().and_then(|value| value.to_str());
    ext.is_some_and(|value| TYPESCRIPT_EXTENSIONS.contains(&value))
}

/// Kind of workspace finding (maps to Nifty lint rule ids).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceFindingKind {
    TypeScriptLargeFile,
}

/// One workspace scan finding.
#[derive(Debug, Clone)]
pub struct WorkspaceFinding {
    pub kind: WorkspaceFindingKind,
    pub path: PathBuf,
    pub line: Option<usize>,
    pub message: String,
}

/// TypeScript sources over `1000` lines.
pub fn scan_large_typescript_files(root: &std::path::Path) -> Vec<WorkspaceFinding> {
    let mut findings = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(should_walk_entry)
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() || !is_typescript_source(entry.path()) {
            continue;
        }
        let content = fs::read_to_string(entry.path()).unwrap_or_default();
        let lines = content.lines().count();
        if lines > MAX_TYPESCRIPT_LINES {
            findings.push(WorkspaceFinding {
                kind: WorkspaceFindingKind::TypeScriptLargeFile,
                path: entry.path().to_path_buf(),
                line: None,
                message: format!("文件过大: {lines} 行 (建议拆分模块)"),
            });
        }
    }
    findings
}
