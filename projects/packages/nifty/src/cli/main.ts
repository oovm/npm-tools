import { runBump } from "./bump.js";
import { runLint } from "./lint.js";
import { runPublish } from "./publish.js";
import { runUpdate } from "./update.js";
import { runUpload } from "./upload.js";

export async function runCli(argv: string[]): Promise<number> {
    const [command, ...rest] = argv;

    if (!command || command === "-h" || command === "--help") {
        printHelp();
        return 0;
    }

    try {
        switch (command) {
            case "update":
                await runUpdate(rest);
                return 0;
            case "lint":
                await runLint(rest, false);
                return 0;
            case "check":
                await runLint(rest, true);
                return 0;
            case "upload":
                await runUpload(rest);
                return 0;
            case "bump":
                await runBump(rest);
                return 0;
            case "publish":
                await runPublish(rest);
                return 0;
            default:
                console.error(`unknown command: ${command}`);
                printHelp();
                return 1;
        }
    } catch (error) {
        console.error(`error: ${error instanceof Error ? error.message : String(error)}`);
        return 1;
    }
}

function printHelp(): void {
    console.log(`nifty — Nifty tooling for hybrid cargo + npm projects

Usage:
  nifty update [-i] [-C cwd]
  nifty lint [--from ref] [--to ref] [-s subject] [-C cwd]
  nifty check [--from ref] [--to ref] [-s subject] [-C cwd]
  nifty upload (--release | --pages | --both) [options]
  nifty bump [patch|minor|major|version] [--dry-run] [-C cwd]
  nifty publish [--dry-run] [--tag <tag>] [--access public] [-C cwd]
`);
}
