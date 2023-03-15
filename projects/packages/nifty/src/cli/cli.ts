import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { createCli } from "@vmz/commander";

import { registerBumpCommand } from "./bump-cmd.js";
import { registerLintCommands } from "./lint-cmd.js";
import { registerPublishCommand } from "./publish-cmd.js";
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
    registerLintCommands(cli);
    registerUploadCommand(cli);
    registerBumpCommand(cli);
    registerPublishCommand(cli);
    registerTrustCommand(cli);

    return cli;
}

export async function runCli(argv: string[]): Promise<number> {
    try {
        return await buildNiftyCli().parse(argv);
    } catch (error) {
        console.error(`error: ${error instanceof Error ? error.message : String(error)}`);
        return 1;
    }
}
