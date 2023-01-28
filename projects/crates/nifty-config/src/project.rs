use std::fs;
use std::path::{Path, PathBuf};

/// Detected project ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// Only `Cargo.toml` found in ancestry.
    Cargo,
    /// Only `package.json` found in ancestry.
    Npm,
    /// Both Cargo and npm manifests found (monorepo or mixed layout).
    Hybrid,
    /// No known manifest found.
    Unknown,
}

/// Layout hints for cargo + npm hybrid repositories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLayout {
    /// Resolved project root (git root when present).
    pub root: PathBuf,
    /// Detected ecosystem kind.
    pub kind: ProjectKind,
    /// Nearest `Cargo.toml` walking upward from the start directory.
    pub cargo_manifest: Option<PathBuf>,
    /// Nearest `package.json` walking upward (skips `node_modules`).
    pub package_manifest: Option<PathBuf>,
    /// Nearest directory whose `Cargo.toml` declares a `[workspace]`.
    pub cargo_workspace_root: Option<PathBuf>,
    /// Nearest directory whose `package.json` declares npm `workspaces`.
    pub npm_workspace_root: Option<PathBuf>,
}

const CARGO_MANIFEST: &str = "Cargo.toml";
const PACKAGE_MANIFEST: &str = "package.json";
const GIT_DIR: &str = ".git";

/// Walk upward and return the nearest `Cargo.toml`.
pub fn find_cargo_manifest(start: &Path) -> Option<PathBuf> {
    find_file_upward(start, CARGO_MANIFEST, false)
}

/// Walk upward and return the nearest `package.json` (skips `node_modules`).
pub fn find_package_manifest(start: &Path) -> Option<PathBuf> {
    find_file_upward(start, PACKAGE_MANIFEST, true)
}

/// Detect cargo/npm/hybrid layout from `start` (usually `cwd`).
pub fn detect_project_layout(start: &Path) -> ProjectLayout {
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());
    let cargo_manifest = find_cargo_manifest(&start);
    let package_manifest = find_package_manifest(&start);
    let cargo_workspace_root = cargo_manifest
        .as_deref()
        .and_then(|manifest| find_cargo_workspace_root(manifest));
    let npm_workspace_root = package_manifest
        .as_deref()
        .and_then(|manifest| find_npm_workspace_root(manifest));
    let kind = classify_project(cargo_manifest.is_some(), package_manifest.is_some());
    let root = find_git_root(&start)
        .or(cargo_workspace_root.clone())
        .or(npm_workspace_root.clone())
        .or_else(|| cargo_manifest.as_ref().and_then(|path| path.parent().map(Path::to_path_buf)))
        .or_else(|| package_manifest.as_ref().and_then(|path| path.parent().map(Path::to_path_buf)))
        .unwrap_or(start);

    ProjectLayout {
        root,
        kind,
        cargo_manifest,
        package_manifest,
        cargo_workspace_root,
        npm_workspace_root,
    }
}

fn classify_project(has_cargo: bool, has_npm: bool) -> ProjectKind {
    match (has_cargo, has_npm) {
        (true, true) => ProjectKind::Hybrid,
        (true, false) => ProjectKind::Cargo,
        (false, true) => ProjectKind::Npm,
        (false, false) => ProjectKind::Unknown,
    }
}

fn find_file_upward(start: &Path, file_name: &str, skip_node_modules: bool) -> Option<PathBuf> {
    let start = start.canonicalize().ok()?;
    let mut here = start.as_path();
    loop {
        if skip_node_modules && is_inside_node_modules(here) {
            here = here.parent()?;
            continue;
        }
        let candidate = here.join(file_name);
        if candidate.is_file() {
            return candidate.canonicalize().ok();
        }
        here = here.parent()?;
    }
}

fn is_inside_node_modules(path: &Path) -> bool {
    path.components().any(|component| component.as_os_str() == "node_modules")
}

fn find_git_root(start: &Path) -> Option<PathBuf> {
    let start = start.canonicalize().ok()?;
    let mut here = start.as_path();
    loop {
        if here.join(GIT_DIR).exists() {
            return here.canonicalize().ok();
        }
        here = here.parent()?;
    }
}

fn find_cargo_workspace_root(manifest: &Path) -> Option<PathBuf> {
    let mut here = manifest.parent()?;
    loop {
        let candidate = here.join(CARGO_MANIFEST);
        if candidate.is_file() && manifest_has_cargo_workspace(&candidate) {
            return here.canonicalize().ok();
        }
        here = here.parent()?;
    }
}

fn find_npm_workspace_root(manifest: &Path) -> Option<PathBuf> {
    let mut here = manifest.parent()?;
    loop {
        let candidate = here.join(PACKAGE_MANIFEST);
        if candidate.is_file() && manifest_has_npm_workspaces(&candidate) {
            return here.canonicalize().ok();
        }
        here = here.parent()?;
    }
}

fn manifest_has_cargo_workspace(manifest: &Path) -> bool {
    fs::read_to_string(manifest)
        .ok()
        .is_some_and(|content| content.lines().any(|line| line.trim() == "[workspace]"))
}

fn manifest_has_npm_workspaces(manifest: &Path) -> bool {
    let content = match fs::read_to_string(manifest) {
        Ok(content) => content,
        Err(_) => return false,
    };
    content.contains("\"workspaces\"")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{detect_project_layout, find_cargo_manifest, find_package_manifest, ProjectKind};

    #[test]
    fn detects_hybrid_repo_from_nested_package() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root");
        let layout = detect_project_layout(&root.join("projects/packages/nifty"));
        assert_eq!(layout.kind, ProjectKind::Hybrid);
        assert!(layout.cargo_manifest.is_some());
        assert!(layout.package_manifest.is_some());
        assert_eq!(layout.root, root);
        assert_eq!(layout.cargo_workspace_root.as_deref(), Some(root.as_path()));
    }

    #[test]
    fn finds_nearest_cargo_manifest_in_crate_dir() {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../nifty-types")
            .canonicalize()
            .expect("nifty-types dir");
        let manifest = find_cargo_manifest(&crate_dir).expect("crate manifest");
        assert!(manifest.ends_with("nifty-types/Cargo.toml"));
    }

    #[test]
    fn finds_repo_package_json_from_crate_dir() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root");
        let crate_dir = root.join("projects/crates/nifty-types");
        let manifest = find_package_manifest(&crate_dir).expect("root package.json");
        assert_eq!(manifest, root.join("package.json"));
    }
}
