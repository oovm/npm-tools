import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const { installNativeFromOptions } = await import("../dist/cli/install-native.js");

/**
 * Platform packages as the workspace declares them: `os` + `cpu` mark a native package and
 * `main` names the addon that must reach the tarball.
 */
const MATRIX = [
    {
        dir: "nifty-win32-x64",
        name: "@doki-land/nifty-win32-x64",
        os: "win32",
        cpu: "x64",
        main: "./lib/index.win32-x64-msvc.node",
    },
    {
        dir: "nifty-linux-x64",
        name: "@doki-land/nifty-linux-x64",
        os: "linux",
        cpu: "x64",
        main: "./lib/index.linux-x64-gnu.node",
    },
    {
        dir: "nifty-linux-arm64",
        name: "@doki-land/nifty-linux-arm64",
        os: "linux",
        cpu: "arm64",
        main: "./lib/index.linux-arm64-gnu.node",
    },
    {
        dir: "nifty-darwin-x64",
        name: "@doki-land/nifty-darwin-x64",
        os: "darwin",
        cpu: "x64",
        main: "./lib/index.darwin-x64.node",
    },
    {
        dir: "nifty-darwin-arm64",
        name: "@doki-land/nifty-darwin-arm64",
        os: "darwin",
        cpu: "arm64",
        main: "./lib/index.darwin-arm64.node",
    },
];

function artifactName(entry) {
    return path.basename(entry.main);
}

function scratch() {
    return fs.mkdtempSync(path.join(os.tmpdir(), "nifty-install-native-"));
}

function writeWorkspace(root, matrix) {
    const packages = path.join(root, "projects", "packages");
    fs.mkdirSync(path.join(root, "projects", "crates"), { recursive: true });
    for (const entry of matrix) {
        const dir = path.join(packages, entry.dir);
        fs.mkdirSync(dir, { recursive: true });
        fs.writeFileSync(
            path.join(dir, "package.json"),
            JSON.stringify({ name: entry.name, version: "0.0.0", main: entry.main, os: [entry.os], cpu: [entry.cpu] }),
        );
    }
    return packages;
}

function stageArtifacts(root, matrix) {
    const artifacts = path.join(root, "native-artifacts");
    fs.mkdirSync(artifacts, { recursive: true });
    for (const entry of matrix) {
        fs.writeFileSync(path.join(artifacts, artifactName(entry)), `built-${entry.dir}`);
    }
    return artifacts;
}

test("stages the addon each platform manifest declares as main", () => {
    const root = scratch();
    try {
        const packages = writeWorkspace(root, MATRIX);
        const artifacts = stageArtifacts(root, MATRIX);
        installNativeFromOptions({ cwd: root, from: artifacts });
        for (const entry of MATRIX) {
            const staged = path.join(packages, entry.dir, "lib", artifactName(entry));
            assert.ok(fs.existsSync(staged), `expected ${entry.dir} to receive ${artifactName(entry)}`);
        }
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test("refuses a staging tree that is missing a matrix platform", () => {
    const root = scratch();
    try {
        const packages = writeWorkspace(root, MATRIX);
        const incomplete = MATRIX.filter((entry) => entry.dir !== "nifty-linux-arm64");
        const artifacts = stageArtifacts(root, incomplete);
        assert.throws(
            () => installNativeFromOptions({ cwd: root, from: artifacts }),
            /@doki-land\/nifty-linux-arm64/,
            "a partial native staging tree must not be reported as ready",
        );
        assert.ok(
            !fs.existsSync(path.join(packages, "nifty-linux-arm64", "lib", "index.linux-arm64-gnu.node")),
            "the missing platform must not be reported as staged",
        );
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test("drops an addon left behind by an earlier staging run", () => {
    const root = scratch();
    try {
        const packages = writeWorkspace(root, MATRIX);
        const stale = path.join(packages, "nifty-win32-x64", "lib", "nifty-win32-x64-msvc.node");
        fs.mkdirSync(path.dirname(stale), { recursive: true });
        fs.writeFileSync(stale, "previous-build");
        const artifacts = stageArtifacts(root, MATRIX);
        installNativeFromOptions({ cwd: root, from: artifacts });
        assert.ok(!fs.existsSync(stale), "a stale addon must not survive next to the manifest entry");
        const staged = path.join(packages, "nifty-win32-x64", "lib", "index.win32-x64-msvc.node");
        assert.strictEqual(fs.readFileSync(staged, "utf8"), "built-nifty-win32-x64");
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test("refuses a platform package that does not declare a .node entry", () => {
    const root = scratch();
    try {
        const broken = MATRIX.map((entry) =>
            entry.dir === "nifty-darwin-arm64" ? { ...entry, main: "./index.js" } : entry,
        );
        writeWorkspace(root, broken);
        const artifacts = stageArtifacts(root, MATRIX);
        assert.throws(
            () => installNativeFromOptions({ cwd: root, from: artifacts }),
            /nifty-darwin-arm64/,
            "a platform package with a non-addon entry is not a staged native package",
        );
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});
