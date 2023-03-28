//! Cargo workspace hygiene scans (ported from `cargo cry` in cargo-tools).

mod scanner;

pub use scanner::{
    scan_doc_spec, scan_integrity, scan_large_files, scan_misplaced, CargoFindingKind,
};
