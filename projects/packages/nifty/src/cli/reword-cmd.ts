import type { Cli, ParsedOptions } from "@vmz/commander";

import { loadNiftyNative } from "../native.js";
import { bootstrapFromOptions } from "./context.js";
import { cwdFrom, flag, str } from "./options.js";

export function registerRewordCommand(cli: Cli): void {
    const reword = cli.command("reword", "cli.cmd.reword");

    reword
        .command("export", "cli.cmd.reword.export")
        .option("--base <ref>", "cli.opt.base")
        .option("--ref <ref>", "cli.opt.ref")
        .option("--path <file>", "cli.opt.path")
        .action(async (options) => cmdRewordExport(options));

    reword
        .command("rewrite", "cli.cmd.reword.rewrite")
        .option("--base <ref>", "cli.opt.base")
        .option("--ref <ref>", "cli.opt.ref")
        .option("--path <file>", "cli.opt.path")
        .option("--dry-run", "cli.opt.dry-run")
        .action(async (options) => cmdRewordRewrite(options));
}

export async function cmdRewordExport(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    const base = requireStr(options, "base");
    const path = str(options, "path") ?? "reword.pending.json";
    const native = loadNiftyNative();
    const report = native.history["reword-export"]({
        cwd: cwdFrom(options),
        base,
        ref: str(options, "ref"),
        path,
    });
    console.log(`export: wrote ${report.count} commit entry(ies) to ${report.path}`);
    return 0;
}

export async function cmdRewordRewrite(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    const base = requireStr(options, "base");
    const path = requireStr(options, "path");
    const dryRun = flag(options, "dry-run");
    const native = loadNiftyNative();
    const report = native.history["reword-rewrite"]({
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
        `rewrote ${report.changes.length} commit object(s); ${report.oldTip} -> ${report.newTip ?? report.oldTip}`,
    );
    for (const change of report.changes) {
        console.log(`  ${change.oldOid}  ${change.newSubject}`);
    }
    return 0;
}

function requireStr(options: ParsedOptions, key: string): string {
    const value = str(options, key);
    if (!value) {
        throw new Error(`missing required --${key}`);
    }
    return value;
}
