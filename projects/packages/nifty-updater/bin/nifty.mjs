#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const crateDir = resolve(here, "../../../crates/nifty-updater");
const args = process.argv.slice(2);

const result = spawnSync("cargo", ["run", "--quiet", "--manifest-path", join(crateDir, "Cargo.toml"), "--", ...args], {
    stdio: "inherit",
    cwd: process.cwd(),
});

process.exit(result.status ?? 1);
