import type { Cli, ParsedOptions } from "@vmz/commander";

import { loadNiftyNative } from "../native.js";
import { authPayload } from "./authArgs.js";
import { trustPayloadFromConfig } from "../config/trustPayload.js";
import { bootstrapFromOptions, loadInvocationConfig, trackAction } from "./context.js";
import { printPublishReport } from "./publish.js";
import { authFromOptions, cwdFrom, flag, str, strList } from "./options.js";

export function registerPublishCommand(cli: Cli): void {
    cli.command("publish", "cli.cmd.publish")
        .option("--dry-run", "cli.opt.dry-run")
        .option("--refresh", "cli.opt.refresh")
        .option("--placeholder", "cli.opt.placeholder")
        .option("--package <name>", "cli.opt.package")
        .option("--tag <tag>", "cli.opt.tag")
        .option("--access <access>", "cli.opt.access")
        .option("--version <version>", "cli.opt.version")
        .option("--otp <code>", "cli.opt.otp")
        .option("--totp-secret <secret>", "cli.opt.totp-secret")
        .option("--npm-token <token>", "cli.opt.npm-token")
        .action(trackAction(async (options) => cmdPublish(options)));
}

/**
 * Collect every `--package` occurrence from the raw argv.
 *
 * The option parser keeps only the last value of a repeated flag, so a request like
 * `--package a --package b` used to collapse to `b` and publish less than asked while
 * still exiting 0. Accumulating from argv keeps the full requested set.
 */
export function requestedPackageNames(argv: string[]): string[] {
    const names: string[] = [];
    for (let index = 0; index < argv.length; index += 1) {
        const arg = argv[index];
        if (arg === "--package") {
            const value = argv[index + 1];
            if (value !== undefined && !value.startsWith("-")) {
                names.push(value);
                index += 1;
            }
            continue;
        }
        if (arg.startsWith("--package=")) {
            const value = arg.slice("--package=".length);
            if (value.length > 0) {
                names.push(value);
            }
        }
    }
    return names;
}

export async function cmdPublish(options: ParsedOptions): Promise<number> {
    await bootstrapFromOptions(options);
    const { config } = await loadInvocationConfig({ cwd: cwdFrom(options), createIfMissing: false });
    const requested = new Set([...requestedPackageNames(process.argv.slice(2)), ...strList(options, "package")]);
    const packages = [...requested];
    const fromConfig = config.publish?.packages ?? [];
    const native = loadNiftyNative();
    const report = native.publisher["publish-workspace"]({
        cwd: cwdFrom(options),
        dryRun: flag(options, "dry-run"),
        refresh: flag(options, "refresh"),
        placeholder: flag(options, "placeholder"),
        tag: str(options, "tag"),
        access: str(options, "access") ?? "public",
        version: str(options, "version"),
        ...(packages.length === 1
            ? { only: packages[0] }
            : packages.length > 1
              ? { packages }
              : fromConfig.length > 0
                ? { packages: fromConfig }
                : {}),
        ...trustPayloadFromConfig(config),
        ...authPayload(authFromOptions(options)),
    });
    printPublishReport(report, flag(options, "dry-run"));
    return 0;
}
