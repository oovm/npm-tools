import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { checkPublishVersions, listPublishableManifests } from "./check-publish-versions.mjs";

/** Write a throwaway workspace whose `projects/packages/<dir>` carries the given manifest. */
function workspace(packages) {
    const root = mkdtempSync(join(tmpdir(), "nifty-publish-versions-"));
    for (const [dir, manifest] of Object.entries(packages)) {
        mkdirSync(join(root, "projects", "packages", dir), { recursive: true });
        writeFileSync(join(root, "projects", "packages", dir, "package.json"), JSON.stringify(manifest), "utf8");
    }
    return root;
}

test("derives the release set from the workspace packages and skips private ones", (t) => {
    const root = workspace({
        nifty: { name: "@doki-land/nifty", version: "0.1.12" },
        "nifty-skills": { name: "@doki-land/nifty-skills", version: "0.1.12" },
        "nifty-linux-x64": { name: "@doki-land/nifty-linux-x64", version: "0.1.12", os: ["linux"], cpu: ["x64"] },
        internal: { name: "@doki-land/internal", version: "0.0.1", private: true },
    });
    t.after(() => rmSync(root, { recursive: true, force: true }));

    const result = checkPublishVersions({ root, version: "0.1.12" });
    assert.equal(result.ok, true, "private packages must not block the tag version");
    assert.equal(result.count, 3);
    assert.deepEqual(result.mismatches, []);
    assert.deepEqual(
        listPublishableManifests(root).map((manifest) => manifest.name),
        ["@doki-land/nifty", "@doki-land/nifty-linux-x64", "@doki-land/nifty-skills"],
    );
});

test("reports every publishable manifest that drifted from the tag version", (t) => {
    const root = workspace({
        nifty: { name: "@doki-land/nifty", version: "0.1.12" },
        "nifty-skills": { name: "@doki-land/nifty-skills", version: "0.1.11" },
        "nifty-linux-x64": { name: "@doki-land/nifty-linux-x64", version: "0.1.10" },
    });
    t.after(() => rmSync(root, { recursive: true, force: true }));

    const result = checkPublishVersions({ root, version: "0.1.12" });
    assert.equal(result.ok, false);
    assert.equal(result.count, 3);
    assert.deepEqual(
        result.mismatches.map((mismatch) => mismatch.manifest),
        ["projects/packages/nifty-linux-x64/package.json", "projects/packages/nifty-skills/package.json"],
    );
});

test("fails when the workspace exposes no publishable package", (t) => {
    const root = workspace({ internal: { name: "@doki-land/internal", version: "0.1.12", private: true } });
    t.after(() => rmSync(root, { recursive: true, force: true }));

    const result = checkPublishVersions({ root, version: "0.1.12" });
    assert.equal(result.ok, false);
    assert.equal(result.error, "no publishable package found");
});

test("fails when a publishable manifest loses its version", (t) => {
    const root = workspace({ nifty: { name: "@doki-land/nifty" } });
    t.after(() => rmSync(root, { recursive: true, force: true }));

    assert.throws(
        () => checkPublishVersions({ root, version: "0.1.12" }),
        /projects\/packages\/nifty\/package\.json must declare both name and version/,
    );
});
