import assert from "node:assert/strict";
import test from "node:test";

const { requestedPackageNames } = await import("../dist/cli/publish-cmd.js");

test("keeps every repeated --package flag", () => {
    const names = requestedPackageNames([
        "publish",
        "--dry-run",
        "--package",
        "@wae/wae",
        "--package",
        "@wae/commander",
        "--version",
        "0.0.1",
    ]);
    assert.deepEqual(names, ["@wae/wae", "@wae/commander"]);
});

test("accepts --package=value", () => {
    assert.deepEqual(requestedPackageNames(["publish", "--package=@wae/wae"]), ["@wae/wae"]);
});

test("ignores a trailing --package without a value", () => {
    assert.deepEqual(requestedPackageNames(["publish", "--package"]), []);
});
