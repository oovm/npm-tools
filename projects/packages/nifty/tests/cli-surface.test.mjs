import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import test from "node:test";

// The commander migration replaced the hand-written argv parsers. These entries must stay
// gone: each of them parsed argv a second time and silently ignored a missing option value.
const LEGACY_ENTRIES = ["runBump", "runPublish", "runTrust", "runUpdate", "runLint", "runUpload"];

const packageRoot = fileURLToPath(new URL("..", import.meta.url));

test("the CLI subpath exposes no hand-written argv parsers", async () => {
    const cli = await import("../dist/cli/index.js");
    for (const name of LEGACY_ENTRIES) {
        assert.equal(cli[name], undefined, `${name} must not be exported`);
    }
    assert.equal(typeof cli.runCli, "function", "runCli remains the CLI entry");
});

test("a missing option value fails through commander instead of bumping", () => {
    let error;
    try {
        execFileSync(process.execPath, ["cli/nifty.mjs", "bump", "--dry-run", "--version"], {
            cwd: packageRoot,
            encoding: "utf8",
        });
    } catch (caught) {
        error = caught;
    }
    assert.ok(error, "the CLI must exit non-zero");
    assert.equal(error.status, 2, "commander rejects the input with its usage exit code");
    assert.match(`${error.stdout}${error.stderr}`, /missing value for `--version`/);
});
