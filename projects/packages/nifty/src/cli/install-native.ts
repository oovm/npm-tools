import { copyFileSync, existsSync, mkdirSync, readdirSync, readFileSync, unlinkSync } from "node:fs";
import { basename, dirname, join, relative, resolve } from "node:path";

import { detectProjectLayout } from "../config/detectProject.js";

export type InstallNativeOptions = {
    /** Downloaded CI artifact tree (walked recursively for .node files). */
    from: string;
    cwd?: string;
};

type NativePackage = {
    /** Directory name under `projects/packages`, used in diagnostics. */
    dirName: string;
    /** Package name from the manifest, used in diagnostics. */
    name: string;
    dir: string;
    /** Manifest `main`, relative to `dir`. */
    main: string;
};

function walkNodes(dir: string): string[] {
    const out: string[] = [];
    if (!existsSync(dir)) {
        return out;
    }
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const full = join(dir, entry.name);
        if (entry.isDirectory()) {
            out.push(...walkNodes(full));
        } else if (entry.isFile() && entry.name.endsWith(".node")) {
            out.push(full);
        }
    }
    return out;
}

/**
 * The platform package manifests are the staging contract: `os` + `cpu` mark a native package,
 * `main` names the addon that has to be in the tarball. Every native package must be staged, so
 * the required set is derived from the workspace instead of a hand-maintained subset.
 */
function listNativePackages(root: string): NativePackage[] {
    const packagesDir = join(root, "projects/packages");
    const packages: NativePackage[] = [];
    for (const entry of readdirSync(packagesDir, { withFileTypes: true })) {
        if (!entry.isDirectory()) {
            continue;
        }
        const dir = join(packagesDir, entry.name);
        const manifestPath = join(dir, "package.json");
        if (!existsSync(manifestPath)) {
            continue;
        }
        const manifest = JSON.parse(readFileSync(manifestPath, "utf8")) as {
            name?: string;
            os?: string[];
            cpu?: string[];
            main?: string;
        };
        const native =
            Array.isArray(manifest.os) && manifest.os.length > 0 && Array.isArray(manifest.cpu) && manifest.cpu.length > 0;
        if (!native) {
            continue;
        }
        const main = manifest.main;
        if (typeof main !== "string" || !main.endsWith(".node")) {
            throw new Error(`install-native: ${entry.name}/package.json must declare its addon as "main", got ${main}`);
        }
        packages.push({ dirName: entry.name, name: manifest.name ?? entry.name, dir, main });
    }
    return packages;
}

function stageNativeBinary(root: string, pkg: NativePackage, source: string): void {
    const dest = join(pkg.dir, pkg.main);
    const libDir = dirname(dest);
    mkdirSync(libDir, { recursive: true });
    // An addon left by an earlier run would ship instead of this build's artifact.
    for (const name of readdirSync(libDir).filter((entry) => entry.endsWith(".node"))) {
        unlinkSync(join(libDir, name));
    }
    copyFileSync(source, dest);
    console.log(`installed ${relative(root, dest)}`);
}

/** Copy matrix .node artifacts into platform package lib dirs for npm publish. */
export function installNativeFromOptions(options: InstallNativeOptions): void {
    const layout = detectProjectLayout(options.cwd ?? process.cwd());
    const artifactRoot = resolve(layout.root, options.from);
    const nodes = walkNodes(artifactRoot);
    if (nodes.length === 0) {
        throw new Error(`install-native: no .node files under ${artifactRoot}`);
    }

    const packages = listNativePackages(layout.root);
    if (packages.length === 0) {
        throw new Error("install-native: no platform package declares os/cpu under projects/packages");
    }

    const byArtifact = new Map(packages.map((pkg) => [basename(pkg.main), pkg]));
    const staged = new Set<string>();
    for (const file of nodes) {
        const artifact = basename(file);
        const pkg = byArtifact.get(artifact);
        if (!pkg) {
            console.log(`skip unmapped artifact ${relative(artifactRoot, file)}`);
            continue;
        }
        if (staged.has(pkg.dirName)) {
            console.log(`skip duplicate artifact ${artifact}`);
            continue;
        }
        stageNativeBinary(layout.root, pkg, file);
        staged.add(pkg.dirName);
    }

    const missing = packages.filter((pkg) => !staged.has(pkg.dirName));
    if (missing.length > 0) {
        throw new Error(
            `install-native: missing native binary for ${missing
                .map((pkg) => `${pkg.name} (${basename(pkg.main)})`)
                .join(", ")}`,
        );
    }

    console.log(`install-native: ${staged.size} native binary(ies) ready for npm publish`);
}
