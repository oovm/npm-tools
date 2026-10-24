import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { createCli, normalizeArgv } from "@vmz/commander";

import { NiftyConfigError } from "../config/loadConfig.js";
import { registerBumpCommand } from "./bump-cmd.js";
import { actionCountSnapshot, resetConfigScope } from "./context.js";
import { registerFormatCommand } from "./format-cmd.js";
import { registerChangeLogsCommand } from "./change-logs-cmd.js";
import { registerInstallNativeCommand } from "./install-native-cmd.js";
import { registerLintCommands } from "./lint-cmd.js";
import { registerPublishCommand } from "./publish-cmd.js";
import { registerRetimeCommand } from "./retime-cmd.js";
import { registerCommitCommand } from "./commit-cmd.js";
import { registerTrustCommand } from "./trust-cmd.js";
import { registerUpdateCommand } from "./update-cmd.js";
import { registerUploadCommand } from "./upload-cmd.js";

function buildNiftyCli() {
    const localesRoot = join(dirname(fileURLToPath(import.meta.url)), "../../locales");
    const cli = createCli("nifty")
        .locales(localesRoot)
        .intro("cli.intro")
        .option("-C, --cwd <dir>", "cli.opt.cwd");

    registerUpdateCommand(cli);
    registerFormatCommand(cli);
    registerLintCommands(cli);
    registerUploadCommand(cli);
    registerInstallNativeCommand(cli);
    registerBumpCommand(cli);
    registerPublishCommand(cli);
    registerTrustCommand(cli);
    registerCommitCommand(cli);
    registerRetimeCommand(cli);
    registerChangeLogsCommand(cli);

    return cli;
}

const packageVersion = readPackageVersion();

function readPackageVersion(): string {
    try {
        const manifest = JSON.parse(readFileSync(new URL("../../package.json", import.meta.url), "utf8")) as {
            version?: string;
        };
        return manifest.version ?? "0.0.0";
    } catch {
        return "0.0.0";
    }
}

function reportError(error: unknown): number {
    if (error instanceof NiftyConfigError) {
        console.error(`error [${error.code}] ${error.configPath}: ${error.args.reason}`);
        if (error.cause !== undefined) {
            const cause = error.cause instanceof Error ? error.cause.message : String(error.cause);
            console.error(`  cause: ${cause}`);
        }
        return 1;
    }
    console.error(`error: ${error instanceof Error ? error.message : String(error)}`);
    return 1;
}

export async function runCli(argv: string[]): Promise<number> {
    const args = normalizeArgv(argv);
    if (args.length === 1 && (args[0] === "--version" || args[0] === "-V")) {
        console.log(packageVersion);
        return 0;
    }

    resetConfigScope();
    const actionsBefore = actionCountSnapshot();
    try {
        const code = await buildNiftyCli().parse(argv);
        return code === 1 && actionCountSnapshot() === actionsBefore ? 2 : code;
    } catch (error) {
        return reportError(error);
    }
}
