//! Workspace-wide hygiene scans beyond cargo-specific rules.

mod scanner;

pub use scanner::{WorkspaceFindingKind, scan_large_typescript_files};
