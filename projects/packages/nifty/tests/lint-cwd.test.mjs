import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const { lintCommits } = await import("../dist/cli/lint.js");

function git(cwd, ...args) {
    return execFileSync("git", args, { cwd, encoding: "utf8" });
}

/** Isolated repository whose own history is the only thing `-C` may lint. */
function probeRepo() {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "nifty-lint-cwd-"));
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

test("commit lint honours -C for an explicit range", async () => {
    const root = probeRepo();
    try {
        const count = await lintCommits(
            { subjects: [], cwd: root, from: "v0.0.0", to: "v0.0.1", commitOnly: true, scanCargo: false },
            false,
        );
        assert.equal(count, 2, "only the probe repository's two invalid commits may be reported");
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});

test("commit lint honours -C without an explicit range", async () => {
    const root = probeRepo();
    try {
        const count = await lintCommits({ subjects: [], cwd: root, commitOnly: true, scanCargo: false }, false);
        assert.equal(count, 2, "the whole probe history must be scanned, not the caller's repository");
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
    }
});
