/**
 * Run nifty from npm when a published CLI exists, else fall back to workspace `pnpm exec nifty`.
 *
 * CI publish/upload jobs should not depend on building the monorepo just to drive release steps.
 * Set NIFTY_CLI_VERSION to pin (e.g. last stable). PUBLISH_VERSION is tried first on tag retries.
 */

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const PACKAGE = "@doki-land/nifty";
const args = process.argv.slice(2);

if (args.length === 0 || args[0] === "-h" || args[0] === "--help") {
    console.log(`usage: node scripts/ci/nifty.mjs <nifty-args...>

Resolves CLI version:
  1. NIFTY_CLI_VERSION
  2. PUBLISH_VERSION when that exact version is on npm (tag retry)
  3. latest published ${PACKAGE}
  4. pnpm exec nifty (bootstrap before first npm release)`);
    process.exit(args.length === 0 ? 1 : 0);
}

function npmView(spec) {
    const result = spawnSync("npm", ["view", spec, "version", "--json"], {
        encoding: "utf8",
        shell: process.platform === "win32",
        stdio: ["ignore", "pipe", "pipe"],
    });
    if (result.status !== 0) {
        return undefined;
    }
    const raw = String(result.stdout ?? "").trim();
    if (!raw || raw === "null" || raw === '""') {
        return undefined;
    }
    try {
        const parsed = JSON.parse(raw);
        if (typeof parsed === "string" && parsed.length > 0) {
            return parsed;
        }
    } catch {
        if (raw.length > 0 && !raw.startsWith("{")) {
            return raw;
        }
    }
    return undefined;
}

function resolveCliVersion() {
    if (process.env.NIFTY_CLI_VERSION?.trim()) {
        return process.env.NIFTY_CLI_VERSION.trim();
    }
    const publishVersion = process.env.PUBLISH_VERSION?.trim();
    const latest = npmView(PACKAGE);
    if (publishVersion) {
        // Tag retry: reuse the CLI version already on npm for this release.
        if (npmView(`${PACKAGE}@${publishVersion}`)) {
            return publishVersion;
        }
        // First publish of a new semver: workspace CLI is newer than registry latest.
        if (latest && latest !== publishVersion) {
            return null;
        }
    }
    return latest;
}

function run(cmd, cmdArgs, opts = {}) {
    const result = spawnSync(cmd, cmdArgs, {
        cwd: opts.cwd ?? ROOT,
        encoding: "utf8",
        shell: process.platform === "win32",
        stdio: "inherit",
        env: process.env,
    });
    if ((result.status ?? 1) !== 0) {
        process.exit(result.status ?? 1);
    }
}

const version = resolveCliVersion();
if (version) {
    console.log(`nifty-ci: ${PACKAGE}@${version} ${args.join(" ")}`);
    run("npx", ["-y", `${PACKAGE}@${version}`, ...args]);
} else {
    console.log(`nifty-ci: workspace nifty ${args.join(" ")} (no published CLI yet)`);
    run("pnpm", ["exec", "nifty", ...args]);
}
