/**
 * Copy downloaded CI native artifacts into projects/packages/nifty-{platform}/lib/.
 *
 * Input tree: NATIVE_ARTIFACT_ROOT (default: native-artifacts/ at repo root).
 * Output: one .node per platform package, ready for nifty publish (npm only).
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

/** @type {Record<string, string>} */
const PACKAGE_BY_ASSET = {
    "nifty-win32-x64-msvc.node": "nifty-win32-x64",
    "nifty-linux-x64-gnu.node": "nifty-linux-x64",
    "nifty-linux-arm64-gnu.node": "nifty-linux-arm64",
    "nifty-darwin-x64.node": "nifty-darwin-x64",
    "nifty-darwin-arm64.node": "nifty-darwin-arm64",
};

/** @param {string} dir */
function walkNodes(dir) {
    /** @type {string[]} */
    const out = [];
    if (!fs.existsSync(dir)) {
        return out;
    }
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            out.push(...walkNodes(full));
        } else if (entry.isFile() && entry.name.endsWith(".node")) {
            out.push(full);
        }
    }
    return out;
}

function main() {
    const artifactRoot = path.resolve(ROOT, process.env.NATIVE_ARTIFACT_ROOT || "native-artifacts");
    const nodes = walkNodes(artifactRoot);
    if (nodes.length === 0) {
        console.error(`install-native-artifacts: no .node files under ${artifactRoot}`);
        process.exit(1);
    }

    let installed = 0;
    for (const file of nodes) {
        const base = path.basename(file);
        const packageDir = PACKAGE_BY_ASSET[base];
        if (!packageDir) {
            console.log(`skip unmapped artifact ${path.relative(artifactRoot, file)}`);
            continue;
        }
        const dest = path.join(ROOT, "projects/packages", packageDir, "lib", base);
        fs.mkdirSync(path.dirname(dest), { recursive: true });
        fs.copyFileSync(file, dest);
        console.log(`installed ${path.relative(ROOT, dest)}`);
        installed += 1;
    }

    if (installed === 0) {
        console.error("install-native-artifacts: no mapped platform .node files installed");
        process.exit(1);
    }

    const required = ["nifty-win32-x64", "nifty-linux-x64", "nifty-darwin-arm64"];
    for (const packageDir of required) {
        const libDir = path.join(ROOT, "projects/packages", packageDir, "lib");
        const hasNode = fs.existsSync(libDir) && fs.readdirSync(libDir).some((name) => name.endsWith(".node"));
        if (!hasNode) {
            console.error(`install-native-artifacts: missing .node under ${packageDir}/lib`);
            process.exit(1);
        }
    }

    console.log(`install-native-artifacts: ${installed} native binary(ies) ready for npm publish`);
}

main();
