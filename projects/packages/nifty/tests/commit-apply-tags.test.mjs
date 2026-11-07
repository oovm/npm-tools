import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const { cmdCommitApply } = await import("../dist/cli/commit-cmd.js");

function git(cwd, ...args) {
    return execFileSync("git", args, { cwd, encoding: "utf8" });
}

/** Probe repository whose `dev` tip is tagged, so a rewrite leaves the tag behind. */
function probeRepo() {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "nifty-apply-tags-"));
    git(root, "init", "-b", "dev");
    git(root, "config", "user.name", "Probe");
    git(root, "config", "user.email", "probe@example.com");
    fs.writeFileSync(path.join(root, "a.txt"), "a");
    git(root, "add", "a.txt");
    git(root, "commit", "-q", "-m", "🎂 Project initialized!");
    git(root, "tag", "v0.0.0");
    git(root, "commit", "-q", "--allow-empty", "-m", "🔖 Add beta");
    git(root, "commit", "-q", "--allow-empty", "-m", "Add gamma without gitmoji");
    git(root, "tag", "v0.0.1");
    return root;
}

function writeMap(root, hash, message) {
    const mapPath = path.join(root, "apply.json");
    fs.writeFileSync(mapPath, JSON.stringify({ version: 1, entries: [{ hash, message }] }), "utf8");
    return mapPath;
}

async function captureLogs(run) {
    const lines = [];
    const original = console.log;
    console.log = (...parts) => lines.push(parts.join(" "));
    try {
        await run();
    } finally {
        console.log = original;
    }
    return lines;
}

test("commit apply warns about tags left on pre-rewrite commits", async () => {
    const root = probeRepo();
    try {
        const tipBefore = git(root, "rev-parse", "dev").trim();
        const tagged = git(root, "rev-parse", "v0.0.1").trim();
        const mapPath = writeMap(root, tagged, "📝 Add gamma without gitmoji");

        const lines = await captureLogs(async () => {
            const code = await cmdCommitApply({ cwd: root, base: "v0.0.0", ref: "dev", path: mapPath });
            assert.equal(code, 0);
        });

        assert.ok(
            lines.some((line) => line === "warning: 1 tag(s) inside the rewritten range were not moved: v0.0.1"),
            "affected tags must be named in a warning",
        );
        assert.notEqual(git(root, "rev-parse", "dev").trim(), tipBefore, "apply must move the branch");
        assert.equal(git(root, "rev-parse", "v0.0.1").trim(), tagged, "apply must not move tags");
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test("commit apply dry-run reports affected tags and keeps the ref", async () => {
    const root = probeRepo();
    try {
        const tipBefore = git(root, "rev-parse", "dev").trim();
        const tagged = git(root, "rev-parse", "v0.0.1").trim();
        const mapPath = writeMap(root, tagged, "📝 Add gamma without gitmoji");

        const lines = await captureLogs(async () => {
            const code = await cmdCommitApply({ cwd: root, base: "v0.0.0", ref: "dev", path: mapPath, "dry-run": true });
            assert.equal(code, 0);
        });

        assert.ok(lines.some((line) => line.includes("inside the rewritten range would keep pointing at pre-rewrite commits: v0.0.1")));
        assert.equal(git(root, "rev-parse", "dev").trim(), tipBefore);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});
