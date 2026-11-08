import { execFileSync, execSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, unlinkSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const napiDir = join(root, "projects/crates/nifty-napi");
const packagesDir = join(root, "projects/packages");

/**
 * Platform package manifests are the artifact contract: `os` + `cpu` identify the target and
 * `main` names the addon that gets published. This script stages exactly that file for the host
 * platform and copies nothing else.
 */
function readPlatformPackages() {
    const packages = [];
    for (const entry of readdirSync(packagesDir, { withFileTypes: true })) {
        if (!entry.isDirectory()) {
            continue;
        }
        const dir = join(packagesDir, entry.name);
        const manifestPath = join(dir, "package.json");
        if (!existsSync(manifestPath)) {
            continue;
        }
        const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
        if (
            !Array.isArray(manifest.os) ||
            manifest.os.length === 0 ||
            !Array.isArray(manifest.cpu) ||
            manifest.cpu.length === 0
        ) {
            continue;
        }
        if (typeof manifest.main !== "string" || !manifest.main.endsWith(".node")) {
            throw new Error(`${entry.name}/package.json must declare its addon as "main", got ${manifest.main}`);
        }
        packages.push({
            dir,
            dirName: entry.name,
            os: manifest.os,
            cpu: manifest.cpu,
            main: manifest.main,
            artifact: basename(manifest.main),
        });
    }
    return packages;
}

function nodeFiles(dir) {
    return existsSync(dir) ? readdirSync(dir).filter((name) => name.endsWith(".node")) : [];
}

/** A `.node` left behind by an earlier run must never be staged as this build's output. */
function clearNodes(dir) {
    for (const name of nodeFiles(dir)) {
        unlinkSync(join(dir, name));
    }
}

const host = readPlatformPackages().filter(
    (pkg) => pkg.os.includes(process.platform) && pkg.cpu.includes(process.arch),
);
if (host.length !== 1) {
    throw new Error(
        `expected exactly one platform package for ${process.platform}-${process.arch}, found ${
            host.map((pkg) => pkg.dirName).join(", ") || "none"
        }`,
    );
}
const target = host[0];

// `napi` runs an unlocked `cargo metadata` of its own before the locked build, so a stale
// `Cargo.lock` would be rewritten and the build would carry on. Check it here, before anything
// compiles, and fail loudly instead. `--cargo-flags=--locked` is not the drift guard: `ci-gate`
// is what pins the published graph, and this check only keeps this build honest.
try {
    execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
        cwd: root,
        encoding: "utf8",
        stdio: ["ignore", "ignore", "pipe"],
    });
} catch (error) {
    console.error("Cargo.lock does not match the workspace manifests.");
    console.error(String(error.stderr ?? "").trim() || error.message);
    console.error("Refresh the lock where the local Oak overlay is off, then rebuild.");
    process.exit(1);
}

clearNodes(napiDir);

execSync("pnpm exec napi build --platform --release --cargo-flags=--locked", {
    cwd: napiDir,
    stdio: "inherit",
    shell: true,
});

const source = join(napiDir, target.artifact);
if (!existsSync(source)) {
    throw new Error(
        `napi build did not produce ${target.artifact} for ${target.dirName} (found: ${
            nodeFiles(napiDir).join(", ") || "no .node files"
        })`,
    );
}
for (const name of nodeFiles(napiDir).filter((name) => name !== target.artifact)) {
    console.warn(`skip unclaimed artifact: ${name}`);
}

const dest = join(target.dir, target.main);
mkdirSync(dirname(dest), { recursive: true });
clearNodes(dirname(dest));
copyFileSync(source, dest);
console.log(`staged ${target.artifact} -> ${dest}`);
