import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

import {
    collectInternalDependencyNames,
    findWorkspaceRoot,
    listWorkspacePackages,
    type PackageManifest,
    type WorkspacePackage,
} from "./workspace.js";

export type PublishOptions = {
    cwd?: string;
    dryRun?: boolean;
    tag?: string;
    access?: string;
};

export type PublishReport = {
    root: string;
    order: string[];
    published: string[];
    skipped: string[];
};

const DEPENDENCY_FIELDS = [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
] as const;

export async function runPublish(argv: string[]): Promise<void> {
    const options = parsePublishArgs(argv);
    const report = publishWorkspace(options);
    printReport(report, options.dryRun);
}

function parsePublishArgs(argv: string[]): PublishOptions {
    let cwd: string | undefined;
    let dryRun = false;
    let tag: string | undefined;
    let access = "public";

    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === "--dry-run") {
            dryRun = true;
        } else if (arg === "-C" || arg === "--cwd") {
            cwd = argv[++i];
        } else if (arg === "--tag") {
            tag = argv[++i];
        } else if (arg === "--access") {
            access = argv[++i];
        } else if (arg === "-h" || arg === "--help") {
            printPublishHelp();
            process.exit(0);
        } else {
            throw new Error(`unknown publish argument: ${arg}`);
        }
    }

    return { cwd, dryRun, tag, access };
}

export function publishWorkspace(options: PublishOptions): PublishReport {
    const root = findWorkspaceRoot(options.cwd ?? process.cwd());
    const packages = listWorkspacePackages(root);
    const publishable = packages.filter((pkg) => !pkg.private);
    const skipped = packages.filter((pkg) => pkg.private).map((pkg) => pkg.name);
    const byName = new Map(packages.map((pkg) => [pkg.name, pkg]));
    const order = sortPackagesForPublish(publishable, byName).map((pkg) => pkg.name);

    const published: string[] = [];
    for (const name of order) {
        const pkg = byName.get(name);
        if (!pkg) {
            throw new Error(`missing workspace package ${name}`);
        }
        publishPackage(pkg, byName, options);
        published.push(name);
    }

    return { root, order, published, skipped };
}

function sortPackagesForPublish(
    packages: WorkspacePackage[],
    byName: Map<string, WorkspacePackage>,
): WorkspacePackage[] {
    const inDegree = new Map<string, number>();
    const dependents = new Map<string, string[]>();

    for (const pkg of packages) {
        inDegree.set(pkg.name, 0);
        dependents.set(pkg.name, []);
    }

    for (const pkg of packages) {
        const deps = collectInternalDependencyNames(pkg, byName);
        for (const dep of deps) {
            if (!inDegree.has(dep)) {
                continue;
            }
            inDegree.set(pkg.name, (inDegree.get(pkg.name) ?? 0) + 1);
            dependents.get(dep)?.push(pkg.name);
        }
    }

    const queue = packages
        .filter((pkg) => (inDegree.get(pkg.name) ?? 0) === 0)
        .map((pkg) => pkg.name)
        .sort();
    const sorted: string[] = [];

    while (queue.length > 0) {
        const name = queue.shift()!;
        sorted.push(name);
        for (const dependent of dependents.get(name) ?? []) {
            const next = (inDegree.get(dependent) ?? 1) - 1;
            inDegree.set(dependent, next);
            if (next === 0) {
                queue.push(dependent);
                queue.sort();
            }
        }
    }

    if (sorted.length !== packages.length) {
        throw new Error("cyclic workspace dependency among npm packages");
    }

    return sorted.map((name) => byName.get(name)!);
}

function publishPackage(
    pkg: WorkspacePackage,
    byName: Map<string, WorkspacePackage>,
    options: PublishOptions,
): void {
    const original = readFileSync(pkg.manifestPath, "utf8");
    const patched = patchManifestForPublish(pkg.manifest, pkg.dir, byName);
    const next = `${JSON.stringify(patched, null, 4)}\n`;

    let restored = false;
    const restore = (): void => {
        if (!restored) {
            writeFileSync(pkg.manifestPath, original, "utf8");
            restored = true;
        }
    };

    try {
        if (next !== original) {
            writeFileSync(pkg.manifestPath, next, "utf8");
        }
        runNpmPublish(pkg.dir, options);
    } finally {
        restore();
    }
}

function patchManifestForPublish(
    manifest: PackageManifest,
    packageDir: string,
    byName: Map<string, WorkspacePackage>,
): PackageManifest {
    const next: PackageManifest = { ...manifest };
    for (const field of DEPENDENCY_FIELDS) {
        const entries = manifest[field];
        if (!entries) {
            continue;
        }
        next[field] = patchDependencyEntries(entries, packageDir, byName);
    }
    return next;
}

function patchDependencyEntries(
    entries: Record<string, string>,
    packageDir: string,
    byName: Map<string, WorkspacePackage>,
): Record<string, string> {
    const next: Record<string, string> = {};
    for (const [name, spec] of Object.entries(entries)) {
        const workspacePkg = resolveWorkspaceDependency(name, spec, packageDir, byName);
        next[name] = workspacePkg ? workspacePkg.version : spec;
    }
    return next;
}

function resolveWorkspaceDependency(
    name: string,
    spec: string,
    packageDir: string,
    byName: Map<string, WorkspacePackage>,
): WorkspacePackage | undefined {
    const direct = byName.get(name);
    if (direct && (spec.startsWith("file:") || spec.startsWith("workspace:"))) {
        return direct;
    }
    if (!spec.startsWith("file:")) {
        return undefined;
    }
    for (const candidate of byName.values()) {
        if (candidate.dir === resolve(packageDir, spec.slice("file:".length))) {
            return candidate;
        }
    }
    return undefined;
}

function runNpmPublish(dir: string, options: PublishOptions): void {
    const args = ["publish"];
    if (options.dryRun) {
        args.push("--dry-run");
    }
    if (options.tag) {
        args.push("--tag", options.tag);
    }
    if (options.access) {
        args.push("--access", options.access);
    }

    const result = spawnSync("npm", args, {
        cwd: dir,
        stdio: "inherit",
        shell: process.platform === "win32",
    });
    if (result.status !== 0) {
        throw new Error(`npm publish failed in ${dir}`);
    }
}

function printReport(report: PublishReport, dryRun?: boolean): void {
    const prefix = dryRun ? "would publish" : "published";
    console.log(`${prefix} workspace ${report.root}`);
    if (report.skipped.length > 0) {
        console.log(`skipped private packages: ${report.skipped.join(", ")}`);
    }
    console.log("publish order:");
    for (const name of report.order) {
        const marker = report.published.includes(name) ? prefix : "pending";
        console.log(`  ${marker}: ${name}`);
    }
}

function printPublishHelp(): void {
    console.log(`nifty publish — publish workspace npm packages in dependency order

Usage:
  nifty publish
  nifty publish --dry-run
  nifty publish --tag next
  nifty publish --access public
  nifty publish -C <cwd>
`);
}
