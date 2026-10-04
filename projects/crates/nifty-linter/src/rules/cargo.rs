use std::path::Path;

use crate::{
    cargo::{CargoFindingKind, scan_doc_spec, scan_integrity, scan_large_files, scan_misplaced},
    context::{LintContext, file_diagnostic},
    rule::{
        LintDiagnostic, RULE_CARGO_DOC_INCLUDE_STR, RULE_CARGO_LARGE_FILE, RULE_CARGO_MISPLACED_ROOT_RS,
        RULE_CARGO_MISPLACED_TEST, RULE_CARGO_MISSING_DOCS, RULE_CARGO_PACKAGE_SECTION, RULE_CARGO_README_CASE,
        RULE_CARGO_README_MISSING, RULE_CARGO_WORKSPACE_DEP, RULE_CARGO_WORKSPACE_INHERIT,
    },
};

pub fn lint_cargo_workspace(ctx: &LintContext, root: &Path) -> Vec<LintDiagnostic> {
    let mut findings = Vec::new();
    findings.extend(scan_integrity(root));
    findings.extend(scan_misplaced(root));
    findings.extend(scan_doc_spec(root));
    findings.extend(scan_large_files(root));

    findings
        .into_iter()
        .filter_map(|finding| {
            let rule_id = rule_id_for_kind(finding.kind);
            file_diagnostic(ctx, rule_id, finding.message, &finding.path, finding.line.map(|line| line as u32))
        })
        .collect()
}

fn rule_id_for_kind(kind: CargoFindingKind) -> &'static str {
    match kind {
        CargoFindingKind::ReadmeCase => RULE_CARGO_README_CASE,
        CargoFindingKind::PackageSection => RULE_CARGO_PACKAGE_SECTION,
        CargoFindingKind::ReadmeMissing => RULE_CARGO_README_MISSING,
        CargoFindingKind::MissingDocs => RULE_CARGO_MISSING_DOCS,
        CargoFindingKind::WorkspaceInherit => RULE_CARGO_WORKSPACE_INHERIT,
        CargoFindingKind::WorkspaceDep => RULE_CARGO_WORKSPACE_DEP,
        CargoFindingKind::DocIncludeStr => RULE_CARGO_DOC_INCLUDE_STR,
        CargoFindingKind::MisplacedTest => RULE_CARGO_MISPLACED_TEST,
        CargoFindingKind::MisplacedRootRs => RULE_CARGO_MISPLACED_ROOT_RS,
        CargoFindingKind::LargeFile => RULE_CARGO_LARGE_FILE,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::lint_cargo_workspace;
    use crate::{
        context::LintContext,
        rule::{RULE_CARGO_MISSING_DOCS, default_rules},
    };

    #[test]
    fn flags_missing_docs_on_member_crate() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../nifty-types").canonicalize().expect("nifty-types");
        let ctx = LintContext::new(vec![], default_rules());
        let diagnostics = lint_cargo_workspace(&ctx, &root);
        assert!(diagnostics.iter().any(|item| item.rule == RULE_CARGO_MISSING_DOCS));
    }
}
