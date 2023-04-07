use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const SKIP_DIRS: &[&str] = &["node_modules", "target", ".git", "dist", ".cache"];

pub fn discover_format_targets(root: &Path) -> Result<Vec<PathBuf>, String> {
    let has_biome = root.join("biome.json").is_file();
    let mut paths = if let Some(includes) = read_biome_includes(root)? {
        expand_includes(root, &includes)?
    } else if has_biome {
        let mut full = Vec::new();
        collect_full_repo(root, &mut full);
        full
    } else {
        discover_default_targets(root)?
    };

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn read_biome_includes(root: &Path) -> Result<Option<Vec<String>>, String> {
    let biome_path = root.join("biome.json");
    if !biome_path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&biome_path)
        .map_err(|err| format!("{}: {err}", biome_path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|err| format!("{}: {err}", biome_path.display()))?;
    let includes = value
        .pointer("/files/includes")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .filter(|items| !items.is_empty());
    Ok(includes)
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

    for name in ["package.json", "biome.json", "oxfmtrc.json"] {
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
        Some("ts" | "mts" | "cts" | "js" | "mjs" | "cjs" | "jsx" | "tsx" | "json" | "jsonc") => true,
        _ => false,
    }
}

pub fn layout_has_js_targets(root: &Path) -> bool {
    discover_format_targets(root)
        .map(|paths| !paths.is_empty())
        .unwrap_or(false)
}
