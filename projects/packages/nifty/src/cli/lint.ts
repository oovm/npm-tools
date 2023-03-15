import { loadNiftyNative } from "../native.js";

const KNOWN_GITMOJIS = ["✨", "🎨", "🚀", "🐛", "🚑", "🔥", "💥", "♻️", "🔧", "📝", "👷", "🧹", "⬆️", "🧪", "🔨", "📦"];

export type LintOptions = {
    subjects: string[];
    from?: string;
    to?: string;
    cwd?: string;
};

type LintDiagnostic = {
    rule: string;
    severity: "error" | "warning";
    message: string;
    subject?: string;
};

export async function runLint(argv: string[], check: boolean): Promise<void> {
    const options = parseLintArgs(argv);
    const errorCount = await lintCommits(options, check);
    if (check && errorCount > 0) {
        throw new Error(`found ${errorCount} lint error(s)`);
    }
}

export async function lintCommits(options: LintOptions, check: boolean): Promise<number> {
    const native = loadNiftyNative();
    const subjects: string[] = [];

    if (options.subjects.length > 0) {
        subjects.push(...options.subjects);
    } else {
        const cwd = options.cwd ?? process.cwd();
        const repoRoot = native.git["discover-root"](cwd);
        const toRef = options.to ?? "HEAD";
        const commits = native.git["collect-commits"](repoRoot, options.from, toRef);
        subjects.push(...commits.map((commit) => commit.subject));
    }

    const diagnostics = subjects.flatMap((subject) => lintSubject(subject, native));
    for (const item of diagnostics) {
        const prefix = item.severity === "error" ? "error" : "warning";
        const suffix = item.subject ? ` (${item.subject})` : "";
        console.log(`${prefix} [${item.rule}] ${item.message}${suffix}`);
    }

    return diagnostics.filter((item) => item.severity === "error").length;
}

function parseLintArgs(argv: string[]): LintOptions {
    const options: LintOptions = { subjects: [] };
    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === "--from") {
            options.from = argv[++i];
        } else if (arg === "--to") {
            options.to = argv[++i];
        } else if (arg === "-C" || arg === "--cwd") {
            options.cwd = argv[++i];
        } else if (arg === "-s" || arg === "--subject") {
            options.subjects.push(argv[++i]);
        }
    }
    return options;
}

function lintSubject(subject: string, native: ReturnType<typeof loadNiftyNative>): LintDiagnostic[] {
    const out: LintDiagnostic[] = [];
    if (!native.gitmoji["validate-subject"](subject)) {
        out.push({
            rule: "gitmoji/subject",
            severity: "error",
            message: "commit subject must start with a known gitmoji followed by a space",
            subject,
        });
        return out;
    }

    const gitmoji = native.gitmoji["leading-gitmoji"](subject);
    if (gitmoji && !KNOWN_GITMOJIS.includes(gitmoji)) {
        out.push({
            rule: "gitmoji/known",
            severity: "error",
            message: `gitmoji ${gitmoji} is not in the Nifty known gitmoji list`,
            subject,
        });
    }

    const body = native.gitmoji["strip-gitmoji"](subject).trim();
    if (!body) {
        out.push({
            rule: "gitmoji/body",
            severity: "warning",
            message: "commit subject body must not be empty after the gitmoji prefix",
            subject,
        });
    }

    if (gitmoji) {
        const expected = native.gitmoji["format-subject"](gitmoji, body);
        if (expected !== subject.trim()) {
            out.push({
                rule: "gitmoji/format",
                severity: "warning",
                message: `subject should be formatted as ${expected}`,
                subject,
            });
        }
    }

    return out;
}
