import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmdirSync, unlinkSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { createJiti } from "jiti";

const { bumpWorkspace } = await createJiti(import.meta.url, { interopDefault: true }).import("../src/cli/bump.ts");

function createWorkspace({ workspaceVersion = true } = {}) {
    const root = mkdtempSync(path.join(os.tmpdir(), "nifty-bump-workspace-"));
    const inheritedCrate = path.join(root, "projects", "crates", "inherited");
    const inlineInheritedCrate = path.join(root, "projects", "crates", "inline-inherited");
    const explicitCrate = path.join(root, "projects", "crates", "explicit");
    const npmPackage = path.join(root, "projects", "packages", "nifty");
    [inheritedCrate, inlineInheritedCrate, explicitCrate, npmPackage].forEach((directory) => mkdirSync(directory, { recursive: true }));

    writeFileSync(
        path.join(root, "Cargo.toml"),
        `[workspace]\nmembers = ["projects/crates/*"]\n${workspaceVersion ? '\n[workspace.package]\nversion = "0.1.2" # shared version\n' : ""}`,
    );
    writeFileSync(
        path.join(inheritedCrate, "Cargo.toml"),
        '[package]\nname = "inherited"\nversion.workspace = true\n',
    );
    writeFileSync(
        path.join(inlineInheritedCrate, "Cargo.toml"),
        '[package]\nname = "inline-inherited"\nversion = { workspace = true }\n',
    );
    writeFileSync(path.join(explicitCrate, "Cargo.toml"), '[package]\nname = "explicit"\nversion = "0.1.2"\n');
    writeFileSync(path.join(npmPackage, "package.json"), `${JSON.stringify({ name: "@example/nifty", version: "0.1.2" })}\n`);
    return { root, inheritedCrate, inlineInheritedCrate, explicitCrate, npmPackage };
}

function removeWorkspace({ root, inheritedCrate, inlineInheritedCrate, explicitCrate, npmPackage }) {
    [
        path.join(root, "Cargo.toml"),
        path.join(inheritedCrate, "Cargo.toml"),
        path.join(inlineInheritedCrate, "Cargo.toml"),
        path.join(explicitCrate, "Cargo.toml"),
        path.join(npmPackage, "package.json"),
    ].forEach((file) => unlinkSync(file));
    [
        npmPackage,
        explicitCrate,
        inlineInheritedCrate,
        inheritedCrate,
        path.join(root, "projects", "packages"),
        path.join(root, "projects", "crates"),
        path.join(root, "projects"),
        root,
    ].forEach((directory) => rmdirSync(directory));
}

test("bumps inherited Cargo workspace versions and explicit members", () => {
    const workspace = createWorkspace();
    try {
        const options = { cwd: workspace.root, kind: "patch" };
        const dryRun = bumpWorkspace({ ...options, dryRun: true });

        assert.equal(dryRun.from, "0.1.2");
        assert.equal(dryRun.to, "0.1.3");
        assert.ok(dryRun.cargo.includes(path.join(workspace.root, "Cargo.toml")));
        assert.ok(dryRun.cargo.includes(path.join(workspace.explicitCrate, "Cargo.toml")));
        assert.ok(!dryRun.cargo.includes(path.join(workspace.inheritedCrate, "Cargo.toml")));
        assert.ok(!dryRun.cargo.includes(path.join(workspace.inlineInheritedCrate, "Cargo.toml")));
        assert.match(readFileSync(path.join(workspace.root, "Cargo.toml"), "utf8"), /version = "0\.1\.2" # shared version/);

        bumpWorkspace(options);
        assert.match(readFileSync(path.join(workspace.root, "Cargo.toml"), "utf8"), /version = "0\.1\.3" # shared version/);
        assert.match(readFileSync(path.join(workspace.inheritedCrate, "Cargo.toml"), "utf8"), /version\.workspace = true/);
        assert.match(readFileSync(path.join(workspace.inlineInheritedCrate, "Cargo.toml"), "utf8"), /version = \{ workspace = true \}/);
        assert.match(readFileSync(path.join(workspace.explicitCrate, "Cargo.toml"), "utf8"), /version = "0\.1\.3"/);
        assert.equal(JSON.parse(readFileSync(path.join(workspace.npmPackage, "package.json"), "utf8")).version, "0.1.3");
    } finally {
        removeWorkspace(workspace);
    }
});

test("rejects inherited Cargo versions without changing the workspace", () => {
    const workspace = createWorkspace({ workspaceVersion: false });
    try {
        const packagePath = path.join(workspace.npmPackage, "package.json");
        assert.throws(() => bumpWorkspace({ cwd: workspace.root, kind: "patch" }), /no \[workspace\.package\]\.version/);
        assert.equal(JSON.parse(readFileSync(packagePath, "utf8")).version, "0.1.2");
        assert.match(readFileSync(path.join(workspace.explicitCrate, "Cargo.toml"), "utf8"), /version = "0\.1\.2"/);
    } finally {
        removeWorkspace(workspace);
    }
});
