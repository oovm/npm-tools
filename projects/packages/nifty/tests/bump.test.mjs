import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmdirSync, unlinkSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const { bumpWorkspace, bumpWorkspaceWithConfig } = await import("../dist/cli/bump.js");

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

function addPackage(root, dirName, manifest) {
    const directory = path.join(root, "projects", "packages", dirName);
    mkdirSync(directory, { recursive: true });
    const manifestPath = path.join(directory, "package.json");
    writeFileSync(manifestPath, `${JSON.stringify(manifest)}\n`);
    return { directory, manifestPath };
}

function removePackage({ directory, manifestPath }) {
    unlinkSync(manifestPath);
    rmdirSync(directory);
}

test("leaves private packages out of the bump", () => {
    const workspace = createWorkspace();
    const app = addPackage(workspace.root, "app", { name: "@example/app", version: "0.0.1", private: true });
    try {
        const options = { cwd: workspace.root, kind: "patch" };
        const report = bumpWorkspace({ ...options, dryRun: true });
        assert.ok(!report.npm.includes(app.manifestPath), "private package must not be bumped");
        assert.equal(report.from, "0.1.2", "private versions must not join the shared version check");

        bumpWorkspace(options);
        assert.deepEqual(JSON.parse(readFileSync(app.manifestPath, "utf8")), { name: "@example/app", version: "0.0.1", private: true });
        assert.equal(JSON.parse(readFileSync(path.join(workspace.npmPackage, "package.json"), "utf8")).version, "0.1.3");
    } finally {
        removePackage(app);
        removeWorkspace(workspace);
    }
});

test("leaves packages outside publish.packages out of the bump", () => {
    const workspace = createWorkspace();
    const tool = addPackage(workspace.root, "tool", { name: "@example/tool", version: "0.1.2" });
    try {
        const options = { cwd: workspace.root, kind: "patch", packages: ["@example/nifty"] };
        const report = bumpWorkspace({ ...options, dryRun: true });
        assert.ok(!report.npm.includes(tool.manifestPath), "package outside publish.packages must not be bumped");

        bumpWorkspace(options);
        assert.equal(JSON.parse(readFileSync(tool.manifestPath, "utf8")).version, "0.1.2");
        assert.equal(JSON.parse(readFileSync(path.join(workspace.root, "projects", "packages", "nifty", "package.json"), "utf8")).version, "0.1.3");
    } finally {
        removePackage(tool);
        removeWorkspace(workspace);
    }
});

test("reads the publish allowlist from nifty.config", async () => {
    const workspace = createWorkspace();
    const tool = addPackage(workspace.root, "tool", { name: "@example/tool", version: "0.1.2" });
    const configPath = path.join(workspace.root, "nifty.config.mts");
    writeFileSync(configPath, 'export default { publish: { packages: ["@example/nifty"] } };\n');
    try {
        const report = await bumpWorkspaceWithConfig({ cwd: workspace.root, kind: "patch", dryRun: true });
        assert.ok(!report.npm.includes(tool.manifestPath), "nifty.config publish.packages must narrow the bump");
    } finally {
        unlinkSync(configPath);
        removePackage(tool);
        removeWorkspace(workspace);
    }
});
