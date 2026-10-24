import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const cliPath = path.join(packageRoot, "cli", "nifty.mjs");
const { loadConfig, NiftyConfigError } = await import("../dist/config/index.js");
const { version } = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));

function createProject(contents) {
    const root = mkdtempSync(path.join(os.tmpdir(), "nifty-config-"));
    writeFileSync(path.join(root, "package.json"), '{"name":"nifty-config-fixture","private":true}\n');
    writeFileSync(path.join(root, "nifty.config.mts"), contents);
    return root;
}

function runNifty(root, args) {
    return spawnSync(process.execPath, [cliPath, ...args], {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, NODE_ENV: "development" },
    });
}

test("loads a project .mts config through native Node", async () => {
    const root = createProject(`import { writeFileSync } from "node:fs";

writeFileSync(new URL("./evaluated.txt", import.meta.url), "yes");

export default async ({ mode }) => ({ githubToken: \`token-\${mode}\` });
`);
    try {
        const loaded = await loadConfig({ cwd: root, createIfMissing: false });
        assert.equal(loaded.config.githubToken, `token-${process.env.NODE_ENV ?? "development"}`);
        assert.ok(existsSync(path.join(root, "evaluated.txt")));
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});

test("help, version and argv errors do not evaluate the config", () => {
    const root = createProject(`import { writeFileSync } from "node:fs";

writeFileSync(new URL("./evaluated.txt", import.meta.url), "yes");

export default {};
`);
    try {
        const help = runNifty(root, ["--help"]);
        assert.equal(help.status, 0);
        assert.equal(existsSync(path.join(root, "evaluated.txt")), false);

        const versionResult = runNifty(root, ["--version"]);
        assert.equal(versionResult.status, 0);
        assert.equal(versionResult.stdout.trim(), version);
        assert.equal(existsSync(path.join(root, "evaluated.txt")), false);

        const invalid = runNifty(root, ["--not-a-command"]);
        assert.equal(invalid.status, 2);
        assert.equal(existsSync(path.join(root, "evaluated.txt")), false);
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});

test("reports config execution failures as structured errors with exit 1", async () => {
    const root = createProject(`throw new Error("boom");
`);
    try {
        await assert.rejects(loadConfig({ cwd: root, createIfMissing: false }), (error) => {
            assert.ok(error instanceof NiftyConfigError);
            assert.equal(error.code, "config-load-failed");
            assert.ok(error.configPath.endsWith("nifty.config.mts"));
            assert.equal(error.cause instanceof Error ? error.cause.message : undefined, "boom");
            return true;
        });

        const result = runNifty(root, ["format"]);
        assert.equal(result.status, 1);
        assert.match(result.stderr, /error \[config-load-failed\]/);
        assert.match(result.stderr, /nifty\.config\.mts/);
        assert.match(result.stderr, /cause: boom/);
    } finally {
        rmSync(root, { recursive: true, force: true });
    }
});
