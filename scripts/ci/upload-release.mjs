/**
 * CI entry: `nifty upload --release --native` (platform .node binaries).
 *
 * 0.0.1 uses this mjs wrapper. 0.0.2+ may call `nifty-uploader` through NAPI directly.
 */

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function main() {
    const passthrough = process.argv.slice(2);
    if (passthrough.includes("-h") || passthrough.includes("--help")) {
        console.log(`upload-release.mjs — upload native .node addons via nifty upload --native

Usage:
  node scripts/ci/upload-release.mjs [--tag vX.Y.Z] [--draft] [--no-generate-notes]

Requires built artifacts under projects/packages/nifty-*/lib/*.node.
`);
        process.exit(0);
    }

    const args = [
        "exec",
        "nifty",
        "upload",
        "--release",
        "--native",
        "--github-action",
        ...passthrough,
    ];
    const result = spawnSync("pnpm", args, {
        cwd: ROOT,
        stdio: "inherit",
        shell: process.platform === "win32",
    });
    process.exit(result.status ?? 1);
}

main();
