import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const { cmdChangeLogsRender } = await import("../dist/cli/change-logs-cmd.js");

function git(cwd, ...args) {
    return execFileSync("git", args, { cwd, encoding: "utf8" });
}

/** Isolated repository whose `-C` directory must own every relative output path. */
function probeRepo() {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "nifty-changelog-path-"));
    git(root, "init", "-b", "dev");
    git(root, "config", "user.name", "Probe");
    git(root, "config", "user.email", "probe@example.com");
    fs.writeFileSync(path.join(root, "a.txt"), "a");
    git(root, "add", "a.txt");
    git(root, "commit", "-q", "-m", "🎂 Project initialized!");
    git(root, "tag", "v0.0.0");
    git(root, "commit", "-q", "--allow-empty", "-m", "🔖 Add beta");
    return root;
}

test("change-logs resolves --releases-dir against -C", async () => {
    const root = probeRepo();
    const dirName = "nifty-cwd-leak-probe";
    const leaked = path.join(process.cwd(), dirName);
    try {
        const code = await cmdChangeLogsRender({
            cwd: root,
            from: "v0.0.0",
            to: "dev",
            repo: "probe/probe",
            "releases-dir": dirName,
            write: true,
        });
        assert.equal(code, 0);

        const written = path.join(root, dirName, "dev.reference.md");
        assert.ok(fs.existsSync(written), "--releases-dir must be resolved inside the repository named by -C");
        assert.ok(!fs.existsSync(leaked), "--releases-dir must not leak into the caller's process cwd");
    } finally {
        fs.rmSync(root, { recursive: true, force: true });
        if (fs.existsSync(leaked)) {
            fs.rmSync(leaked, { recursive: true, force: true });
        }
    }
});
