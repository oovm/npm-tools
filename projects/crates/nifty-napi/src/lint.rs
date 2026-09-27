//! Node-API bindings for Nifty lint/check.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use nifty_linter::{run_lint, LintOptions as CoreLintOptions, LintReport as CoreLintReport, RuleConfig, RuleSeverity};

#[napi(object)]
pub struct LintRuleConfig {
    pub id: String,
    pub enabled: Option<bool>,
    pub severity: Option<String>,
}

#[napi(object)]
pub struct LintRunOptions {
    pub cwd: Option<String>,
    pub from_ref: Option<String>,
    pub to_ref: Option<String>,
    pub subjects: Option<Vec<String>>,
    pub rules: Option<Vec<LintRuleConfig>>,
    pub scan_cargo: Option<bool>,
    pub commit_only: Option<bool>,
    pub check: Option<bool>,
}

#[napi(object)]
pub struct LintDiagnosticNapi {
    pub rule: String,
    pub severity: String,
    pub message: String,
    pub subject: Option<String>,
    pub hash: Option<String>,
    pub path: Option<String>,
    pub line: Option<u32>,
}

#[napi(object)]
pub struct LintReportNapi {
    pub diagnostics: Vec<LintDiagnosticNapi>,
    pub error_count: u32,
    pub warning_count: u32,
}

fn map_err<T>(result: std::result::Result<T, String>) -> Result<T> {
    result.map_err(|message| Error::from_reason(message))
}

fn map_rules(rules: Option<Vec<LintRuleConfig>>) -> Option<Vec<RuleConfig>> {
    rules.map(|items| {
        items
            .into_iter()
            .map(|item| RuleConfig {
                id: item.id,
                enabled: item.enabled.unwrap_or(true),
                severity: parse_severity(item.severity.as_deref()),
            })
            .collect()
    })
}

fn parse_severity(raw: Option<&str>) -> RuleSeverity {
    match raw.unwrap_or("error").to_ascii_lowercase().as_str() {
        "warning" | "warn" => RuleSeverity::Warning,
        "info" => RuleSeverity::Info,
        _ => RuleSeverity::Error,
    }
}

fn severity_name(severity: RuleSeverity) -> &'static str {
    match severity {
        RuleSeverity::Error => "error",
        RuleSeverity::Warning => "warning",
        RuleSeverity::Info => "info",
    }
}

fn to_napi_report(report: CoreLintReport) -> LintReportNapi {
    LintReportNapi {
        error_count: report.error_count() as u32,
        warning_count: report.warning_count() as u32,
        diagnostics: report
            .diagnostics
            .into_iter()
            .map(|item| LintDiagnosticNapi {
                rule: item.rule,
                severity: severity_name(item.severity).to_string(),
                message: item.message,
                subject: item.subject,
                hash: item.hash,
                path: item.path,
                line: item.line,
            })
            .collect(),
    }
}

fn core_options(options: LintRunOptions) -> CoreLintOptions {
    CoreLintOptions {
        repo_root: None,
        cwd: options.cwd.map(std::path::PathBuf::from),
        from_ref: options.from_ref,
        to_ref: options.to_ref,
        subjects: options.subjects.unwrap_or_default(),
        rules: map_rules(options.rules),
        scan_cargo: options.scan_cargo,
        commit_only: options.commit_only,
    }
}

#[napi]
pub fn lint_run(options: LintRunOptions) -> Result<LintReportNapi> {
    let core = core_options(options);
    map_err(run_lint(core).map(to_napi_report))
}

#[napi]
pub fn lint_check(options: LintRunOptions) -> Result<LintReportNapi> {
    let core = core_options(options);
    map_err(run_lint(core).map(to_napi_report))
}
