mod cargo;
mod engine;
mod oak;
mod oxc;
mod style;
mod walk;

use std::fs;
use std::path::{Path, PathBuf};

use nifty_config::{detect_project_layout, ProjectKind};

pub use engine::{format_source, format_source_with_options};
pub use style::{FormatStyleOptions, default_format_options, resolve_format_options};
/// One formatted file outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatFileResult {
    pub changed: bool,
    pub output: String,
}

/// Workspace format summary.
#[derive(Debug, Clone, Default)]
pub struct FormatReport {
    pub formatted: usize,
    pub unchanged: usize,
    pub errors: Vec<String>,
}

/// Options for [`run_format`].
#[derive(Debug, Clone, Default)]
pub struct RunFormatOptions {
    pub cwd: Option<PathBuf>,
    pub check: bool,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub rust: Option<bool>,
    pub javascript: Option<bool>,
    pub style: Option<FormatStyleOptions>,
}

pub type Result<T> = std::result::Result<T, String>;

/// Format JS/TS/JSX/TSX via Oak (legacy `oxc_formatter` only until Oak coverage closes) and Rust via `cargo fmt`.
pub fn run_format(options: RunFormatOptions) -> Result<FormatReport> {
    let cwd = options
        .cwd
        .clone()
        .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
    let layout = detect_project_layout(&cwd);
    let mut report = FormatReport::default();

    let rust_enabled = options.rust.unwrap_or(matches!(layout.kind, ProjectKind::Cargo | ProjectKind::Hybrid));
    if rust_enabled && matches!(layout.kind, ProjectKind::Cargo | ProjectKind::Hybrid) {
        let cargo_root = layout
            .cargo_workspace_root
            .clone()
            .or(layout.cargo_manifest.as_ref().and_then(|manifest| manifest.parent().map(Path::to_path_buf)))
            .unwrap_or_else(|| layout.root.clone());
        cargo::apply_cargo_fmt(&mut report, &cargo_root, options.check);
    }

    let javascript_enabled = options.javascript.unwrap_or(true);
    if javascript_enabled
        && (matches!(layout.kind, ProjectKind::Npm | ProjectKind::Hybrid)
            || walk::layout_has_js_targets(&layout.root))
    {
        let discover = walk::DiscoverOptions {
            includes: options.includes.clone(),
            excludes: options.excludes.clone(),
        };
        let format_options = resolve_format_options(options.style.as_ref());
        for path in walk::discover_format_targets(&layout.root, &discover)? {
            format_path(&path, options.check, &mut report, &format_options)?;
        }
    }

    Ok(report)
}

fn format_path(
    path: &Path,
    check: bool,
    report: &mut FormatReport,
    options: &oxc_formatter::JsFormatOptions,
) -> Result<()> {
    let source = fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let result = engine::format_source_with_options(path, &source, options.clone())?;

    if !result.changed {
        report.unchanged += 1;
        return Ok(());
    }

    if check {
        report.errors.push(format!("{}: would be reformatted", path.display()));
        return Ok(());
    }

    fs::write(path, &result.output).map_err(|err| format!("{}: {err}", path.display()))?;
    report.formatted += 1;
    println!("formatted {}", path.display());
    Ok(())
}
