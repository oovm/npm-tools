use crate::context::{CommitInput, LintContext};
use crate::rule::{default_rules, LintDiagnostic, RuleConfig, RuleSeverity};
use crate::rules::{lint_cargo_workspace, lint_commit};
use nifty_config::{detect_project_layout, ProjectKind};

pub type Result<T> = std::result::Result<T, String>;

/// Run lint rules and return diagnostics.
pub fn run_lint(options: LintOptions) -> Result<LintReport> {
    let subject_only = is_subject_only(&options);
    let rules = merge_rules(options.rules.clone());
    let commits = load_commits(&options, subject_only)?;
    let ctx = LintContext::new(commits, rules);
    let mut diagnostics = ctx
        .commits
        .iter()
        .flat_map(|commit| lint_commit(&ctx, commit))
        .collect::<Vec<_>>();

    if should_scan_cargo(&options, subject_only) {
        if let Some(root) = resolve_cargo_scan_root(&options) {
            diagnostics.extend(lint_cargo_workspace(&ctx, &root));
        }
    }

    Ok(LintReport { diagnostics })
}

/// Run lint rules and fail when any error-level diagnostic is found.
pub fn run_check(options: LintOptions) -> Result<LintReport> {
    let report = run_lint(options)?;
    if report.has_errors() {
        return Err(format!("found {} lint error(s)", report.error_count()));
    }
    Ok(report)
}

fn merge_rules(custom: Option<Vec<RuleConfig>>) -> Vec<RuleConfig> {
    let Some(custom) = custom else {
        return default_rules();
    };

    let mut merged = default_rules();
    for override_rule in custom {
        if let Some(existing) = merged.iter_mut().find(|rule| rule.id == override_rule.id) {
            *existing = override_rule;
        } else {
            merged.push(override_rule);
        }
    }
    merged
}

fn load_commits(options: &LintOptions, subject_only: bool) -> Result<Vec<CommitInput>> {
    let mut commits = options.subjects.iter().cloned().map(CommitInput::subject_only).collect::<Vec<_>>();

    if subject_only {
        return Ok(commits);
    }

    if options.repo_root.is_some() || options.from_ref.is_some() || options.to_ref.is_some() {
        let repo_root = options
            .repo_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().expect("current dir"));
        commits.extend(crate::git::load_commits(
            &repo_root,
            options.from_ref.as_deref(),
            options.to_ref.as_deref(),
        )?);
    }

    if commits.is_empty() {
        let start = options
            .repo_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        if let Ok(root) = nifty_git::discover_root(&start) {
            commits.extend(crate::git::load_commits(&root, None, Some("HEAD"))?);
        }
    }

    Ok(commits)
}

fn is_subject_only(options: &LintOptions) -> bool {
    !options.subjects.is_empty()
        && options.from_ref.is_none()
        && options.to_ref.is_none()
        && options.repo_root.is_none()
}

fn should_scan_cargo(options: &LintOptions, subject_only: bool) -> bool {
    if options.scan_cargo == Some(false) || subject_only {
        return false;
    }
    options.scan_cargo.unwrap_or(true)
}

fn resolve_cargo_scan_root(options: &LintOptions) -> Option<std::path::PathBuf> {
    let start = options
        .repo_root
        .clone()
        .or_else(|| options.cwd.clone())
        .or_else(|| std::env::current_dir().ok())?;
    let layout = detect_project_layout(&start);
    match layout.kind {
        ProjectKind::Cargo | ProjectKind::Hybrid => layout.cargo_workspace_root.or(Some(layout.root)),
        ProjectKind::Npm | ProjectKind::Unknown => None,
    }
}

/// Lint execution options.
#[derive(Debug, Clone, Default)]
pub struct LintOptions {
    pub repo_root: Option<std::path::PathBuf>,
    pub cwd: Option<std::path::PathBuf>,
    pub from_ref: Option<String>,
    pub to_ref: Option<String>,
    pub subjects: Vec<String>,
    pub rules: Option<Vec<RuleConfig>>,
    /// Run cargo workspace hygiene rules (ported from `cargo cry`). Default: true unless `--subject` only.
    pub scan_cargo: Option<bool>,
}

/// Lint output summary.
#[derive(Debug, Clone, Default)]
pub struct LintReport {
    pub diagnostics: Vec<LintDiagnostic>,
}

impl LintReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|item| item.is_error())
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|item| item.is_error()).count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|item| item.severity == RuleSeverity::Warning)
            .count()
    }

    pub fn print_human(&self) {
        if self.diagnostics.is_empty() {
            println!("lint: no issues found");
            return;
        }
        for item in &self.diagnostics {
            let location = match (&item.path, item.line) {
                (Some(path), Some(line)) => format!("{path}:{line}"),
                (Some(path), None) => path.clone(),
                _ => "-".to_string(),
            };
            let hash = item.hash.as_deref().unwrap_or("-");
            let subject = item.subject.as_deref().unwrap_or("-");
            if item.subject.is_some() {
                println!(
                    "[{:?}] {} ({}) {}",
                    item.severity, item.rule, hash, subject
                );
            } else {
                println!("[{:?}] {} {}", item.severity, item.rule, location);
            }
            println!("  {}", item.message);
        }
        println!(
            "lint: {} error(s), {} warning(s)",
            self.error_count(),
            self.warning_count()
        );
    }
}

pub type LintResult = LintReport;
