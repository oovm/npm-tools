import { execFileSync, execSync } from "node:child_process";
import { copyFileSync, mkdirSync, readdirSync, unlinkSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const napiDir = join(root, "projects/crates/nifty-napi");

/** @type {Record<string, { packageDir: string, fileName: string }>} */
const ARTIFACTS = {
    "index.win32-x64-msvc.node": {
        packageDir: "nifty-win32-x64",
        fileName: "nifty-win32-x64-msvc.node",
    },
    "index.linux-x64-gnu.node": {
        packageDir: "nifty-linux-x64",
        fileName: "nifty-linux-x64-gnu.node",
    },
    "index.darwin-x64.node": {
        packageDir: "nifty-darwin-x64",
        fileName: "nifty-darwin-x64.node",
    },
    "index.darwin-arm64.node": {
        packageDir: "nifty-darwin-arm64",
        fileName: "nifty-darwin-arm64.node",
    },
    "index.linux-arm64-gnu.node": {
        packageDir: "nifty-linux-arm64",
        fileName: "nifty-linux-arm64-gnu.node",
    },
};

function findBuiltNodes(dir) {
    return readdirSync(dir).filter((name) => name.endsWith(".node"));
}

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

execSync("pnpm exec napi build --platform --release --cargo-flags=--locked", {
    cwd: napiDir,
    stdio: "inherit",
    shell: true,
});

const built = findBuiltNodes(napiDir);
if (built.length === 0) {
    throw new Error(`No .node artifacts under ${napiDir}`);
}

let copied = 0;
for (const name of built) {
    const mapping = ARTIFACTS[name];
    if (!mapping) {
        console.warn(`skip unmapped artifact: ${name}`);
        continue;
    }
    const libDir = join(root, "projects/packages", mapping.packageDir, "lib");
    mkdirSync(libDir, { recursive: true });
    for (const existing of readdirSync(libDir).filter((entry) => entry.endsWith(".node"))) {
        unlinkSync(join(libDir, existing));
    }
    const dest = join(libDir, mapping.fileName);
    copyFileSync(join(napiDir, name), dest);
    console.log(`copied ${name} -> ${dest}`);
    copied += 1;
}

if (copied === 0) {
    throw new Error(`Built nodes (${built.join(", ")}) did not match any platform package mapping`);
}
