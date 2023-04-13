import type { Cli, ParsedOptions } from "@vmz/commander";

import { loadNiftyNative } from "../native.js";
import { bootstrapFromOptions } from "./context.js";
import { lintCommits, type LintOptions } from "./lint.js";
import { cwdFrom, flag, str, strList } from "./options.js";

export function registerCommitCommand(cli: Cli): void {
    const commit = cli.command("commit", "cli.cmd.commit");

    commit
        .command("export", "cli.cmd.commit.export")
        .option("--base <ref>", "cli.opt.base")
        .option("--ref <ref>", "cli.opt.ref")
        .option("--path <file>", "cli.opt.path")
        .action(async (options) => cmdCommitExport(options));

    commit
        .command("apply", "cli.cmd.commit.apply")
        .option("--base <ref>", "cli.opt.base")
        .option("--ref <ref>", "cli.opt.ref")
        .option("--path <file>", "cli.opt.path")
        .option("--dry-run", "cli.opt.dry-run")
        .action(async (options) => cmdCommitApply(options));

    commit
        .command("scan", "cli.cmd.commit.scan")
        .option("--from <ref>", "cli.opt.from")
        .option("--to <ref>", "cli.opt.to")
        .option("-s, --subject <text>...", "cli.opt.subject")
        .action(async (options) => cmdCommitScan(options));

    commit
        .command("audit", "cli.cmd.commit.audit")
        .option("--from <ref>", "cli.opt.from")
        .option("--to <ref>", "cli.opt.to")
        .option("-s, --subject <text>...", "cli.opt.subject")
        .action(async (options) => cmdCommitAudit(options));
}

export async function cmdCommitExport(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    const base = requireStr(options, "base");
    const path = str(options, "path") ?? "commit.pending.json";
    const native = loadNiftyNative();
    const report = native.history["commit-export"]({
        cwd: cwdFrom(options),
        base,
        ref: str(options, "ref"),
        path,
    });
    console.log(`export: wrote ${report.count} commit entry(ies) to ${report.path}`);
    return 0;
}

export async function cmdCommitApply(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    const base = requireStr(options, "base");
    const path = requireStr(options, "path");
    const dryRun = flag(options, "dry-run");
    const native = loadNiftyNative();
    const report = native.history["commit-apply"]({
        cwd: cwdFrom(options),
        base,
        ref: str(options, "ref"),
        path,
        dryRun,
    });

    if (dryRun) {
        if (report.changes.length === 0) {
            console.log("dry-run: no commit objects need rewriting");
            return 0;
        }
        console.log(`dry-run: ${report.changes.length} commit object(s) would be rewritten on ${report.refName}\n`);
        for (const change of report.changes) {
            console.log(`${change.oldOid}  ${change.oldSubject}`);
            console.log(`     ->  ${change.newSubject}`);
            if (change.parentsRelinked && !change.messageChanged) {
                console.log("     (parent chain relink only)");
            }
            console.log();
        }
        return 0;
    }

    console.log(
        `applied ${report.changes.length} commit object(s); ${report.oldTip} -> ${report.newTip ?? report.oldTip}`,
    );
    for (const change of report.changes) {
        console.log(`  ${change.oldOid}  ${change.newSubject}`);
    }
    return 0;
}

export async function cmdCommitScan(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    await lintCommits(commitLintOptions(options), false, true);
    return 0;
}

export async function cmdCommitAudit(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    const errorCount = await lintCommits(commitLintOptions(options), true, true);
    if (errorCount > 0) {
        throw new Error(`found ${errorCount} commit audit error(s)`);
    }
    return 0;
}

function commitLintOptions(options: ParsedOptions): LintOptions {
    return {
        subjects: strList(options, "subject"),
        from: str(options, "from"),
        to: str(options, "to"),
        cwd: cwdFrom(options),
        scanCargo: false,
        commitOnly: true,
    };
}

function requireStr(options: ParsedOptions, key: string): string {
    const value = str(options, key);
    if (!value) {
        throw new Error(`missing required --${key}`);
    }
    return value;
}
