//! Workspace-wide hygiene scans beyond cargo-specific rules.

mod scanner;

pub use scanner::{scan_large_typescript_files, WorkspaceFindingKind};
