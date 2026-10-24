import type { Cli, ParsedOptions } from "@vmz/commander";

import { bootstrapFromOptions, trackAction } from "./context.js";
import { formatWorkspace } from "./format.js";
import { cwdFrom, flag } from "./options.js";

export function registerFormatCommand(cli: Cli): void {
    cli.command("format", "cli.cmd.format")
        .option("--check", "cli.opt.check")
        .action(trackAction(async (options) => cmdFormat(options)));
}

export async function cmdFormat(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options, { command: "format" });
    const exitCode = await formatWorkspace({
        check: flag(options, "check"),
        cwd: cwdFrom(options),
    });
    if (exitCode !== 0) {
        throw new Error("format check failed");
    }
    return 0;
}
