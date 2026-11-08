import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

const { listPlatformShards, loadNiftyNative } = await import("../dist/native.js");

/** Native packages are the workspace shards: a manifest with `os` + `cpu`. */
function workspaceShardNames() {
    const packagesDir = path.resolve("..");
    return readdirSync(packagesDir, { withFileTypes: true })
        .filter((entry) => entry.isDirectory())
        .map((entry) => path.join(packagesDir, entry.name, "package.json"))
        .filter((manifestPath) => existsSync(manifestPath))
        .map((manifestPath) => JSON.parse(readFileSync(manifestPath, "utf8")))
        .filter((manifest) => Array.isArray(manifest.os) && Array.isArray(manifest.cpu))
        .map((manifest) => manifest.name);
}

test("derives platform shards from the workspace shard manifests", () => {
    const workspaceShards = workspaceShardNames();
    const shards = listPlatformShards();

    for (const shard of shards) {
        assert.ok(workspaceShards.includes(shard.name), `${shard.name} must be a workspace native package`);
        assert.ok(
            shard.name.endsWith(`${shard.os[0]}-${shard.cpu[0]}`),
            `${shard.name} must name the platform it declares in its manifest`,
        );
    }

    const host = `${process.platform}-${process.arch}`;
    assert.deepEqual(
        shards.find((shard) => shard.name.endsWith(host)),
        { name: `@doki-land/nifty-${host}`, os: [process.platform], cpu: [process.arch] },
    );
});

test("loads the native binding from the matching shard", () => {
    const known = loadNiftyNative().gitmoji["known-gitmojis"]();
    assert.ok(Array.isArray(known) && known.includes("🐛"));
});
