import { resolve } from "node:path";

import type { ActionHandler, ParsedOptions } from "@vmz/commander";

import type { LoadConfigOptions, LoadedNiftyConfig, NiftyConfigEnv } from "../config/index.js";
import { loadConfig } from "../config/loadConfig.js";
import { cwdFrom } from "./options.js";

let invocationConfig: { key: string; value: Promise<LoadedNiftyConfig> } | undefined;
let actionCount = 0;

/** Start a fresh config snapshot for one CLI invocation. */
export function resetConfigScope(): void {
    invocationConfig = undefined;
    actionCount = 0;
}

export function actionCountSnapshot(): number {
    return actionCount;
}

/** Count a dispatched command action so argv errors can be told apart from command failures. */
export function trackAction(handler: ActionHandler): ActionHandler {
    return async (...args) => {
        actionCount += 1;
        return await handler(...args);
    };
}

/** Load config once per invocation and environment tuple (one command sees one config snapshot). */
export function loadInvocationConfig(options: LoadConfigOptions = {}): Promise<LoadedNiftyConfig> {
    const key = [
        options.cwd ?? process.cwd(),
        options.configFile ?? "",
        options.env?.command ?? "",
        options.env?.mode ?? "",
    ].join("\u0000");
    if (invocationConfig?.key === key) {
        return invocationConfig.value;
    }
    const value = loadConfig(options);
    invocationConfig = { key, value };
    return value;
}

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
    await loadInvocationConfig({ cwd, createIfMissing: !dryRun });
}

/** Commander action hook: load config from parsed global/command flags. */
export async function bootstrapFromOptions(
    options: ParsedOptions,
    env?: Partial<NiftyConfigEnv>,
): Promise<void> {
    const cwd = cwdFrom(options) ?? process.cwd();
    const dryRun = options["dry-run"] === true;
    await loadInvocationConfig({ cwd, createIfMissing: !dryRun, ...(env ? { env } : {}) });
}
