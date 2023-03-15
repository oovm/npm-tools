import type { Cli, ParsedOptions } from "@vmz/commander";

import { loadNiftyNative } from "../native.js";
import { authPayload } from "./authArgs.js";
import { bootstrapFromOptions } from "./context.js";
import { printPublishReport } from "./publish.js";
import { authFromOptions, cwdFrom, flag, str } from "./options.js";

export function registerPublishCommand(cli: Cli): void {
    cli.command("publish", "cli.cmd.publish")
        .option("--dry-run", "cli.opt.dry-run")
        .option("--refresh", "cli.opt.refresh")
        .option("--tag <tag>", "cli.opt.tag")
        .option("--access <access>", "cli.opt.access")
        .option("--otp <code>", "cli.opt.otp")
        .option("--totp-secret <secret>", "cli.opt.totp-secret")
        .option("--npm-token <token>", "cli.opt.npm-token")
        .action(async (options) => cmdPublish(options));
}

export async function cmdPublish(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    const native = loadNiftyNative();
    const report = native.publisher["publish-workspace"]({
        cwd: cwdFrom(options),
        dryRun: flag(options, "dry-run"),
        refresh: flag(options, "refresh"),
        tag: str(options, "tag"),
        access: str(options, "access") ?? "public",
        ...authPayload(authFromOptions(options)),
    });
    printPublishReport(report, flag(options, "dry-run"));
    return 0;
}
