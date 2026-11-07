import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const { installNativeFromOptions } = await import("../dist/cli/install-native.js");

/** Built addon basenames and the platform package each one belongs to. */
const ASSETS = [
    ["nifty-win32-x64-msvc.node", "nifty-win32-x64"],
    ["nifty-linux-x64-gnu.node", "nifty-linux-x64"],
    ["nifty-linux-arm64-gnu.node", "nifty-linux-arm64"],
    ["nifty-darwin-x64.node", "nifty-darwin-x64"],
    ["nifty-darwin-arm64.node", "nifty-darwin-arm64"],
];

function scratch() {
    return fs.mkdtempSync(path.join(os.tmpdir(), "nifty-install-native-"));
}

function stageArtifacts(root, assets) {
    const artifacts = path.join(root, "native-artifacts");
    fs.mkdirSync(artifacts, { recursive: true });
    for (const [asset] of assets) {
        fs.writeFileSync(path.join(artifacts, asset), "");
    }
    return artifacts;
}

test("stages every platform in the publish matrix", () => {
    const root = scratch();
    try {
        const artifacts = stageArtifacts(root, ASSETS);
        installNativeFromOptions({ cwd: root, from: artifacts });
        for (const [asset, packageDir] of ASSETS) {
            const staged = path.join(root, "projects/packages", packageDir, "lib", asset);
            assert.ok(fs.existsSync(staged), `expected ${packageDir} to receive ${asset}`);
        }
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test("refuses a staging tree that is missing a matrix platform", () => {
    const root = scratch();
    try {
        const incomplete = ASSETS.filter(([asset]) => asset !== "nifty-linux-arm64-gnu.node");
        const artifacts = stageArtifacts(root, incomplete);
        assert.throws(
            () => installNativeFromOptions({ cwd: root, from: artifacts }),
            /nifty-linux-arm64/,
            "a partial native staging tree must not be reported as ready",
        );
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test("refuses a staging tree that is missing one desktop platform", () => {
    const root = scratch();
    try {
        const incomplete = ASSETS.filter(([asset]) => asset !== "nifty-darwin-x64.node");
        const artifacts = stageArtifacts(root, incomplete);
        assert.throws(
            () => installNativeFromOptions({ cwd: root, from: artifacts }),
            /nifty-darwin-x64/,
            "a partial native staging tree must not be reported as ready",
        );
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});
