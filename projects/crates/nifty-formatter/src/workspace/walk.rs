use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &["node_modules", "target", ".git", "dist", ".cache"];

#[derive(Debug, Clone, Default)]
pub struct DiscoverOptions {
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

pub fn discover_format_targets(root: &Path, options: &DiscoverOptions) -> Result<Vec<PathBuf>, String> {
    let mut paths = if let Some(includes) = options.includes.as_ref().filter(|items| !items.is_empty()) {
        expand_includes(root, includes)?
    } else {
        discover_default_targets(root)?
    };

    if let Some(excludes) = options.excludes.as_ref().filter(|items| !items.is_empty()) {
        paths.retain(|path| !is_excluded(root, path, excludes));
    }

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn expand_includes(root: &Path, includes: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for pattern in includes {
        if pattern.ends_with("/**") {
            let dir = root.join(pattern.trim_end_matches("/**"));
            if dir.is_dir() {
                collect_tree(&dir, &mut paths);
            }
            continue;
        }
        if pattern.contains("**/metadata.json") {
            continue;
        }
        let candidate = root.join(pattern);
        if candidate.is_file() && is_format_target(&candidate) {
            paths.push(candidate);
        }
    }
    Ok(paths)
}

fn discover_default_targets(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();

    for segment in [
        "scripts",
        "projects/packages",
        "projects/conformance",
        "projects/dashboard",
    ] {
        let dir = root.join(segment);
        if dir.is_dir() {
            collect_tree(&dir, &mut paths);
        }
    }

    collect_root_sources(root, &mut paths);

    for name in ["package.json", "nifty.config.ts"] {
        let candidate = root.join(name);
        if candidate.is_file() && is_format_target(&candidate) {
            paths.push(candidate);
        }
    }

    if paths.is_empty() {
        collect_full_repo(root, &mut paths);
    }

    Ok(paths)
}

fn collect_problem_metadata(root: &Path, out: &mut Vec<PathBuf>) {
    let problems = root.join("projects").join("problems");
    if !problems.is_dir() {
        return;
    }
    for entry in WalkDir::new(&problems)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.iter().any(|skip| skip == &name)
        })
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() == "metadata.json" {
            out.push(entry.path().to_path_buf());
        }
    }
}

fn collect_root_sources(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && is_format_target(&path) {
            out.push(path);
        }
    }
}

fn collect_full_repo(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.iter().any(|skip| skip == &name)
        })
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() && is_format_target(entry.path()) {
            out.push(entry.path().to_path_buf());
        }
    }
}

fn collect_tree(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !SKIP_DIRS.iter().any(|skip| skip == &name)
        })
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path().to_path_buf();
        if is_format_target(&path) {
            out.push(path);
        }
    }
}

fn is_format_target(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        // Oak format currently accepts JS/TS extensions (not `.json` / `.jsonc`).
        Some("ts" | "mts" | "cts" | "js" | "mjs" | "cjs" | "jsx" | "tsx") => true,
        _ => false,
    }
}

pub fn layout_has_js_targets(root: &Path) -> bool {
    discover_format_targets(root, &DiscoverOptions::default())
        .map(|paths| !paths.is_empty())
        .unwrap_or(false)
}

fn is_excluded(root: &Path, path: &Path, excludes: &[String]) -> bool {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    excludes.iter().any(|pattern| glob_matches(&relative, pattern))
}

fn glob_matches(relative: &str, pattern: &str) -> bool {
    if pattern.ends_with("/**") {
        let prefix = pattern.trim_end_matches("/**");
        return relative == prefix || relative.starts_with(&format!("{prefix}/"));
    }
    relative == pattern
}
