import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const { cmdCommitExport } = await import("../dist/cli/commit-cmd.js");

function git(cwd, ...args) {
    return execFileSync("git", args, { cwd, encoding: "utf8" });
}

/** Probe repository with a tagged base commit, so `--base` has a stable anchor. */
function probeRepo() {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "nifty-commit-path-"));
    git(root, "init", "-b", "dev");
    git(root, "config", "user.name", "Probe");
    git(root, "config", "user.email", "probe@example.com");
    fs.writeFileSync(path.join(root, "a.txt"), "a");
    git(root, "add", "a.txt");
    git(root, "commit", "-q", "-m", "🎂 Project initialized!");
    git(root, "tag", "v0.0.0");
    git(root, "commit", "-q", "--allow-empty", "-m", "✨ Add alpha");
    git(root, "commit", "-q", "--allow-empty", "-m", "🔖 Add beta");
    return root;
}

test("commit export resolves --path against -C instead of the process cwd", async () => {
    const root = probeRepo();
    const fileName = "probe-export.json";
    const processCwdCopy = path.join(process.cwd(), fileName);
    try {
        const code = await cmdCommitExport({ cwd: root, base: "v0.0.0", ref: "dev", path: fileName });
        assert.equal(code, 0);

        const written = path.join(root, fileName);
        assert.ok(fs.existsSync(written), "--path must land inside the repository named by -C");
        assert.ok(!fs.existsSync(processCwdCopy), "--path must not leak into the caller's process cwd");

        const document = JSON.parse(fs.readFileSync(written, "utf8"));
        assert.equal(document.entries.length, 2);
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
        fs.rmSync(processCwdCopy, { force: true });
    }
});
