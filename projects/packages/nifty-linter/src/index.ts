export type LintSeverity = "error" | "warning" | "info";

export type LintRuleConfig = {
    id: string;
    enabled?: boolean;
    severity?: LintSeverity;
};

export type LintDiagnostic = {
    rule: string;
    severity: LintSeverity;
    message: string;
    subject?: string;
    hash?: string;
};

export type LintOptions = {
    cwd?: string;
    fromRef?: string;
    toRef?: string;
    subjects?: string[];
    rules?: LintRuleConfig[];
    /** When true, throw if any error-level diagnostic is found. */
    check?: boolean;
};

export type LintReport = {
    diagnostics: LintDiagnostic[];
    errorCount: number;
    warningCount: number;
};

export const DEFAULT_RULES: LintRuleConfig[] = [
    { id: "gitmoji/subject", severity: "error" },
    { id: "gitmoji/known", severity: "error" },
    { id: "gitmoji/body", severity: "warning" },
    { id: "gitmoji/format", severity: "warning" },
];

const KNOWN_GITMOJIS = ["✨", "🎨", "🚀", "🐛", "🚑", "🔥", "💥", "♻️", "🔧", "📝", "👷", "🧹", "⬆️", "🧪", "🔨", "📦"];

function leadingGitmoji(subject: string): string | undefined {
    for (const emoji of KNOWN_GITMOJIS) {
        if (subject.startsWith(`${emoji} `) || subject.startsWith(`${emoji}\uFE0F `)) {
            return emoji;
        }
    }
    return undefined;
}

function stripGitmoji(subject: string): string {
    const emoji = leadingGitmoji(subject);
    if (!emoji) {
        return subject.trim();
    }
    return subject.slice(emoji.length).replace(/^\uFE0F?/, "").trimStart();
}

function formatSubject(gitmoji: string, body: string): string {
    const trimmed = body.trim();
    return trimmed ? `${gitmoji} ${trimmed}` : gitmoji;
}

function ruleEnabled(rules: LintRuleConfig[], id: string): LintSeverity | undefined {
    const rule = rules.find((item) => item.id === id);
    if (!rule || rule.enabled === false) {
        return undefined;
    }
    return rule.severity ?? "error";
}

function lintSubject(subject: string, rules: LintRuleConfig[]): LintDiagnostic[] {
    const out: LintDiagnostic[] = [];
    const push = (rule: string, severity: LintSeverity, message: string) => {
        out.push({ rule, severity, message, subject });
    };

    const gitmoji = leadingGitmoji(subject);
    if (!gitmoji) {
        const severity = ruleEnabled(rules, "gitmoji/subject");
        if (severity) {
            push("gitmoji/subject", severity, "commit subject must start with a known gitmoji followed by a space");
        }
        return out;
    }

    const knownSeverity = ruleEnabled(rules, "gitmoji/known");
    if (knownSeverity && !KNOWN_GITMOJIS.includes(gitmoji)) {
        push("gitmoji/known", knownSeverity, `gitmoji \`${gitmoji}\` is not in the Nifty known gitmoji list`);
    }

    const body = stripGitmoji(subject);
    const bodySeverity = ruleEnabled(rules, "gitmoji/body");
    if (bodySeverity && !body.trim()) {
        push("gitmoji/body", bodySeverity, "commit subject body must not be empty after the gitmoji prefix");
    }

    const formatSeverity = ruleEnabled(rules, "gitmoji/format");
    if (formatSeverity) {
        const expected = formatSubject(gitmoji, body);
        if (expected !== subject.trim()) {
            push("gitmoji/format", formatSeverity, `subject should be formatted as \`${expected}\``);
        }
    }

    return out;
}

/** Lint commit subjects with Nifty gitmoji rules. */
export function lintSubjects(subjects: string[], rules: LintRuleConfig[] = DEFAULT_RULES): LintReport {
    const diagnostics = subjects.flatMap((subject) => lintSubject(subject, rules));
    return summarize(diagnostics);
}

function summarize(diagnostics: LintDiagnostic[]): LintReport {
    return {
        diagnostics,
        errorCount: diagnostics.filter((item) => item.severity === "error").length,
        warningCount: diagnostics.filter((item) => item.severity === "warning").length,
    };
}

/** Run `nifty lint` / `nifty check` via the Rust CLI, or lint inline subjects in JS. */
export async function lint(options: LintOptions = {}): Promise<LintReport> {
    if (options.subjects?.length) {
        const report = lintSubjects(options.subjects, options.rules ?? DEFAULT_RULES);
        if (options.check && report.errorCount > 0) {
            throw new Error(`found ${report.errorCount} lint error(s)`);
        }
        return report;
    }

    const { spawnSync } = await import("node:child_process");
    const { dirname, join } = await import("node:path");
    const { fileURLToPath } = await import("node:url");

    const crateManifest = join(dirname(fileURLToPath(import.meta.url)), "../../../crates/nifty-updater/Cargo.toml");
    const cwd = options.cwd ?? process.cwd();
    const args = ["run", "--quiet", "--manifest-path", crateManifest, "--", options.check ? "check" : "lint"];
    if (options.fromRef) {
        args.push("--from", options.fromRef);
    }
    if (options.toRef) {
        args.push("--to", options.toRef);
    }
    args.push("-C", cwd);

    const result = spawnSync("cargo", args, { cwd, encoding: "utf8" });
    if (result.status !== 0) {
        throw new Error(result.stderr?.trim() || "nifty lint failed");
    }

    return { diagnostics: [], errorCount: 0, warningCount: 0 };
}

export { lint as check };
