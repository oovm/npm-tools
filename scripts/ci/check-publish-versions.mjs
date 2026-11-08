/**
 * Release gate: every publishable workspace package must carry the tag version.
 *
 * The candidate set is a workspace fact — `projects/packages/*` minus `private` — not a
 * hand-maintained name list. Deriving it here means a newly added package cannot silently
 * skip the check, and a removed one cannot leave a stale name behind.
 *
 * Usage: node scripts/ci/check-publish-versions.mjs --version=<X.Y.Z> [--root=<dir>]
 */

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

/** Published manifests in `root`, sorted by package name. */
export function listPublishableManifests(root) {
    const packagesDir = join(root, "projects", "packages");
    if (!existsSync(packagesDir)) {
        throw new Error(`no projects/packages directory under ${root}`);
    }
    const manifests = [];
    for (const entry of readdirSync(packagesDir, { withFileTypes: true })) {
        if (!entry.isDirectory() || entry.name === "node_modules") {
            continue;
        }
        const manifestPath = join(packagesDir, entry.name, "package.json");
        if (!existsSync(manifestPath)) {
            continue;
        }
        const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
        if (manifest.private === true) {
            continue;
        }
        if (!manifest.name || !manifest.version) {
            throw new Error(`${toPosix(relative(root, manifestPath))} must declare both name and version`);
        }
        manifests.push({
            name: manifest.name,
            version: manifest.version,
            manifest: toPosix(relative(root, manifestPath)),
        });
    }
    manifests.sort((left, right) => left.name.localeCompare(right.name));
    return manifests;
}

/**
 * Compare the release version with every publishable manifest.
 *
 * @returns {{ ok: boolean, count: number, mismatches: Array<{name: string, version: string, manifest: string}>, error: string|null }}
 */
export function checkPublishVersions({ root, version }) {
    if (!version) {
        throw new Error("release version is required");
    }
    const manifests = listPublishableManifests(root);
    if (manifests.length === 0) {
        return { ok: false, count: 0, mismatches: [], error: "no publishable package found" };
    }
    const mismatches = manifests.filter((manifest) => manifest.version !== version);
    return { ok: mismatches.length === 0, count: manifests.length, mismatches, error: null };
}

function toPosix(path) {
    return path.replace(/\\/g, "/");
}

function optionValue(argv, name) {
    const prefix = `--${name}=`;
    const hit = argv.find((arg) => arg.startsWith(prefix));
    return hit === undefined ? undefined : hit.slice(prefix.length);
}

function fail(message) {
    console.error(`check-publish-versions: ${message}`);
    process.exit(1);
}

function main() {
    const argv = process.argv.slice(2);
    const version = optionValue(argv, "version");
    if (!version) {
        fail("--version=<X.Y.Z> is required");
    }
    const root = optionValue(argv, "root") ?? ".";

    let result;
    try {
        result = checkPublishVersions({ root, version });
    } catch (error) {
        fail(error instanceof Error ? error.message : String(error));
    }
    if (result.error) {
        fail(result.error);
    }
    if (!result.ok) {
        for (const mismatch of result.mismatches) {
            console.error(`${mismatch.manifest} version ${mismatch.version} != tag ${version}`);
        }
        fail(`${result.mismatches.length} of ${result.count} publishable packages do not match the release version`);
    }
    console.log(`release version ${version} matches ${result.count} publishable packages`);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
    main();
}
