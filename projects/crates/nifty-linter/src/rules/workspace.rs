use std::path::Path;

use crate::context::{file_diagnostic, LintContext};
use crate::rule::{LintDiagnostic, RULE_TYPESCRIPT_LARGE_FILE};
use crate::workspace::{scan_large_typescript_files, WorkspaceFindingKind};

pub fn lint_typescript_workspace(ctx: &LintContext, root: &Path) -> Vec<LintDiagnostic> {
    scan_large_typescript_files(root)
        .into_iter()
        .filter_map(|finding| {
            let rule_id = match finding.kind {
                WorkspaceFindingKind::TypeScriptLargeFile => RULE_TYPESCRIPT_LARGE_FILE,
            };
            file_diagnostic(
                ctx,
                rule_id,
                finding.message,
                &finding.path,
                finding.line.map(|line| line as u32),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::lint_typescript_workspace;
    use crate::context::LintContext;
    use crate::rule::{default_rules, RULE_TYPESCRIPT_LARGE_FILE};

    #[test]
    fn flags_typescript_sources_over_line_limit() {
        let root = std::env::temp_dir().join(format!(
            "nifty-linter-ts-large-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp dir");
        let body = "export const x = 1;\n".repeat(1001);
        fs::write(root.join("big.ts"), body).expect("write ts");

        let ctx = LintContext::new(vec![], default_rules());
        let diagnostics = lint_typescript_workspace(&ctx, &root);
        assert!(
            diagnostics
                .iter()
                .any(|item| item.rule == RULE_TYPESCRIPT_LARGE_FILE),
            "expected typescript/large-file diagnostic"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
