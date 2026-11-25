import type { Cli, ParsedOptions } from "@vmz/commander";

import { trackAction } from "./context.js";
import {
    findLoginRoot,
    openInBrowser,
    readSecrets,
    resolveRegistry,
    startWebLogin,
    totpCode,
    totpSecondsLeft,
    totpSecretFrom,
    waitForLoginToken,
    whoami,
    writeNpmToken,
} from "./login.js";
import { cwdFrom, flag, str } from "./options.js";

const DEFAULT_TIMEOUT_SECONDS = 300;

export function registerLoginCommand(cli: Cli): void {
    cli.command("login", "cli.cmd.login")
        .option("--registry <url>", "cli.opt.registry")
        .option("--totp-secret <secret>", "cli.opt.totp-secret")
        .option("--timeout <seconds>", "cli.opt.login-timeout")
        .option("--no-browser", "cli.opt.no-browser")
        .action(trackAction(async (options) => cmdLogin(options)));
}

export async function cmdLogin(options: ParsedOptions): Promise<number> {
    const cwd = cwdFrom(options) ?? process.cwd();
    const root = findLoginRoot(cwd);
    const registry = resolveRegistry(str(options, "registry"));
    const secrets = readSecrets(root);
    const totpSecret = str(options, "totp-secret") ?? totpSecretFrom(secrets);

    const session = await startWebLogin(registry);
    console.log(`login ${registry}`);
    console.log(`  open: ${session.loginUrl}`);
    if (!flag(options, "no-browser")) {
        openInBrowser(session.loginUrl);
    }
    if (totpSecret !== undefined) {
        console.log(`  totp: ${totpCode(totpSecret)} (${totpSecondsLeft()}s left)`);
    }
    console.log("  waiting for browser approval...");

    const timeoutMs = timeoutSeconds(options) * 1000;
    const token = await waitForLoginToken(session.doneUrl, { timeoutMs });
    const username = await whoami(registry, token);
    const path = writeNpmToken(root, token);
    console.log(`  token: ${path}`);
    console.log(`logged in as ${username}`);
    return 0;
}

function timeoutSeconds(options: ParsedOptions): number {
    const raw = str(options, "timeout");
    if (raw === undefined) {
        return DEFAULT_TIMEOUT_SECONDS;
    }
    const seconds = Number(raw);
    if (!Number.isFinite(seconds) || seconds <= 0) {
        throw new Error(`invalid --timeout "${raw}": expected a positive number of seconds`);
    }
    return seconds;
}
