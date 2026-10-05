import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

import { findWorkspaceRoot } from "./workspace.js";

export type BumpKind = "patch" | "minor" | "major";

export type BumpOptions = {
    /** Explicit target version. When set, overrides `kind`. */
    version?: string;
    kind?: BumpKind;
    cwd?: string;
    dryRun?: boolean;
};

export type BumpReport = {
    root: string;
    from: string;
    to: string;
    kind?: BumpKind;
    cargo: string[];
    npm: string[];
};

const DEFAULT_KIND: BumpKind = "patch";

export async function runBump(argv: string[]): Promise < void> {
    const options = parseBumpArgs(argv);
    const report = bumpWorkspace(options);
    printBumpReport(report, options.dryRun);
}

function parseBumpArgs(argv: string[]): BumpOptions {
    let version: string | undefined;
    let kind: BumpKind = DEFAULT_KIND;
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
        } else if (arg === "--patch") {
            kind = "patch";
        } else if (arg === "--minor") {
            kind = "minor";
        } else if (arg === "--major") {
            kind = "major";
        } else if (arg === "-h" || arg === "--help") {
            printBumpHelp();
            process.exit(0);
        } else if (!arg.startsWith("-")) {
            if (isBumpKind(arg)) {
                kind = arg;
            } else {
                version = arg;
            }
        }
    }

    if (version) {
        assertVersion(version);
    }
    return { version, kind, cwd, dryRun };
}

function isBumpKind(value: string): value is BumpKind {
    return value === "patch" || value === "minor" || value === "major";
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

    const cargoPaths = collectCargoManifests(cratesDir);
    const npmPaths = collectPackageManifests(packagesDir);
    const from = readWorkspaceVersion(cargoPaths, npmPaths);
    const to = options.version ?? bumpSemver(from, options.kind ?? DEFAULT_KIND);
    assertVersion(to);

    const cargo = cargoPaths.map((path) => bumpCargoManifest(path, to, options.dryRun));
    const npm = npmPaths.map((path) => bumpPackageManifest(path, to, options.dryRun));

    return {
        root,
        from,
        to,
        kind: options.version ? undefined : options.kind ?? DEFAULT_KIND,
        cargo,
        npm,
    };
}

function readWorkspaceVersion(cargoPaths: string[], npmPaths: string[]): string {
    const versions = new Set < string> ();
    for (const path of cargoPaths) {
        versions.add(readCargoVersion(path));
    }
    for (const path of npmPaths) {
        versions.add(readPackageVersion(path));
    }
    if (versions.size === 0) {
        throw new Error("no crate or package versions found in workspace");
    }
    if (versions.size > 1) {
        throw new Error(`workspace versions are out of sync: ${[...versions].sort().join(", ")}`);
    }
    return[...versions][0];
}

function readCargoVersion(path: string): string {
    const original = readFileSync(path, "utf8");
    const match = original.match(/^version\s*=\s*"([^"]*)"/m);
    if (!match) {
        throw new Error(`no [package].version found in ${path}`);
    }
    return match[1];
}

function readPackageVersion(path: string): string {
    const parsed = JSON.parse(readFileSync(path, "utf8"))as { version?: string };
    if (!parsed.version) {
        throw new Error(`no version field in ${path}`);
    }
    return parsed.version;
}

export function bumpSemver(version: string, kind: BumpKind): string {
    const match = version.match(/^(\d+)\.(\d+)\.(\d+)(?:-.*)?$/);
    if (!match) {
        throw new Error(`cannot bump non-semver version ${version}`);
    }
    let major = Number(match[1]);
    let minor = Number(match[2]);
    let patch = Number(match[3]);

    switch (kind) {
        case "patch":
            patch += 1;
            break;
        case "minor":
            minor += 1;
            patch = 0;
            break;
        case "major":
            major += 1;
            minor = 0;
            patch = 0;
            break;
    }

    return `${major}.${minor}.${patch}`;
}

function collectCargoManifests(cratesDir: string): string[] {
    if (!existsSync(cratesDir)) {
        return[];
    }
    return readdirSync(cratesDir, { withFileTypes: true })
        .filter((entry) => entry.isDirectory())
        .map((entry) => join(cratesDir, entry.name, "Cargo.toml"))
        .filter((path) => existsSync(path));
}

function collectPackageManifests(packagesDir: string): string[] {
    if (!existsSync(packagesDir)) {
        return[];
    }
    return readdirSync(packagesDir, { withFileTypes: true })
        .filter((entry) => entry.isDirectory() && entry.name !== "node_modules")
        .map((entry) => join(packagesDir, entry.name, "package.json"))
        .filter((path) => existsSync(path));
}

function bumpCargoManifest(path: string, version: string, dryRun?: boolean) : string {
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

function bumpPackageManifest(path: string, version: string, dryRun?: boolean) : string {
    const original = readFileSync(path, "utf8");
    const parsed = JSON.parse(original)as { version?: string };
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

export function printBumpReport(report: BumpReport, dryRun?: boolean) : void {
    const prefix = dryRun ? "would bump" : "bumped";
    const kind = report.kind ? ` (${report.kind})` : "";
    console.log(`${prefix} workspace ${report.root} ${report.from} -> ${report.to}${kind}`);
    for (const path of report.cargo) {
        console.log(`  cargo: ${path}`);
    }
    for (const path of report.npm) {
        console.log(`  npm: ${path}`);
    }
}

function printBumpHelp(): void {
    console.log(`nifty bump — align all crate and package versions in the workspace

Usage:
  nifty bump                 # bump patch (default)
  nifty bump patch
  nifty bump minor
  nifty bump major
  nifty bump 1.2.3           # set explicit version
  nifty bump --version 1.2.3
  nifty bump --dry-run
  nifty bump -C <cwd>
`);
}
