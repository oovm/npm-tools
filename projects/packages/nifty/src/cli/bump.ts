import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { findWorkspaceRoot } from "./workspace.js";

export type BumpOptions = {
    version: string;
    cwd?: string;
    dryRun?: boolean;
};

export type BumpReport = {
    root: string;
    version: string;
    cargo: string[];
    npm: string[];
};

const DEFAULT_VERSION = "0.0.0";

export async function runBump(argv: string[]): Promise<void> {
    const options = parseBumpArgs(argv);
    const report = bumpWorkspace(options);
    printReport(report, options.dryRun);
}

function parseBumpArgs(argv: string[]): BumpOptions {
    let version = DEFAULT_VERSION;
    let cwd: string | undefined;
    let dryRun = false;

    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === "--dry-run") {
            dryRun = true;
        } else if (arg === "-C" || arg === "--cwd") {
            cwd = argv[++i];
        } else if (arg === "--version" || arg === "-v") {
            version = argv[++i];
        } else if (arg === "-h" || arg === "--help") {
            printBumpHelp();
            process.exit(0);
        } else if (!arg.startsWith("-")) {
            version = arg;
        }
    }

    assertVersion(version);
    return { version, cwd, dryRun };
}

function assertVersion(version: string): void {
    if (!/^\d+\.\d+\.\d+(-[\w.-]+)?$/.test(version)) {
        throw new Error(`invalid version ${version}, expected semver like 0.0.0`);
    }
}

export function bumpWorkspace(options: BumpOptions): BumpReport {
    const root = findWorkspaceRoot(options.cwd ?? process.cwd());
    const cratesDir = join(root, "projects", "crates");
    const packagesDir = join(root, "projects", "packages");

    const cargo = collectCargoManifests(cratesDir).map((path) => bumpCargoManifest(path, options.version, options.dryRun));
    const npm = collectPackageManifests(packagesDir).map((path) => bumpPackageManifest(path, options.version, options.dryRun));

    return { root, version: options.version, cargo, npm };
}

function collectCargoManifests(cratesDir: string): string[] {
    if (!existsSync(cratesDir)) {
        return [];
    }
    return readdirSync(cratesDir, { withFileTypes: true })
        .filter((entry) => entry.isDirectory())
        .map((entry) => join(cratesDir, entry.name, "Cargo.toml"))
        .filter((path) => existsSync(path));
}

function collectPackageManifests(packagesDir: string): string[] {
    if (!existsSync(packagesDir)) {
        return [];
    }
    return readdirSync(packagesDir, { withFileTypes: true })
        .filter((entry) => entry.isDirectory() && entry.name !== "node_modules")
        .map((entry) => join(packagesDir, entry.name, "package.json"))
        .filter((path) => existsSync(path));
}

function bumpCargoManifest(path: string, version: string, dryRun?: boolean): string {
    const original = readFileSync(path, "utf8");
    const next = original.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
    if (next === original) {
        throw new Error(`no [package].version found in ${path}`);
    }
    if (!dryRun) {
        writeFileSync(path, next, "utf8");
    }
    return path;
}

function bumpPackageManifest(path: string, version: string, dryRun?: boolean): string {
    const original = readFileSync(path, "utf8");
    const parsed = JSON.parse(original) as { version?: string };
    if (!parsed.version) {
        throw new Error(`no version field in ${path}`);
    }
    parsed.version = version;
    const next = `${JSON.stringify(parsed, null, 4)}\n`;
    if (!dryRun) {
        writeFileSync(path, next, "utf8");
    }
    return path;
}

function printReport(report: BumpReport, dryRun?: boolean): void {
    const prefix = dryRun ? "would bump" : "bumped";
    console.log(`${prefix} workspace ${report.root} to ${report.version}`);
    for (const path of report.cargo) {
        console.log(`  cargo: ${path}`);
    }
    for (const path of report.npm) {
        console.log(`  npm: ${path}`);
    }
}

function printBumpHelp(): void {
    console.log(`nifty bump [version] — set all crate and package versions in the workspace

Usage:
  nifty bump                 # default 0.0.0
  nifty bump 0.0.0
  nifty bump --version 1.2.3
  nifty bump --dry-run
  nifty bump -C <cwd>
`);
}
