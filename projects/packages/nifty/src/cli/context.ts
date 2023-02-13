import { resolve } from "node:path";

import { loadConfig } from "../config/loadConfig.js";

/** Parse shared CLI flags used across nifty commands. */
export function parseCliContext(argv: string[]): { cwd: string; dryRun: boolean } {
    let cwd = process.cwd();
    let dryRun = false;

    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === "--dry-run") {
            dryRun = true;
        } else if (arg === "-C" || arg === "--cwd") {
            cwd = resolve(argv[++i] ?? cwd);
        }
    }

    return { cwd, dryRun };
}

/** Ensure `nifty.config.*` exists before running a command (skip create on `--dry-run`). */
export async function bootstrapNiftyConfig(argv: string[]): Promise<void> {
    const { cwd, dryRun } = parseCliContext(argv);
    await loadConfig({ cwd, createIfMissing: !dryRun });
}
