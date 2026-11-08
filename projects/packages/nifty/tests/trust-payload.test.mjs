import assert from "node:assert/strict";
import test from "node:test";

const { trustPayloadFromConfig } = await import("../dist/config/trustPayload.js");

test("maps configured trust fields onto native publisher options", () => {
    assert.deepEqual(
        trustPayloadFromConfig({
            trust: { npm: { repo: "oovm/npm-tools", file: "publish-npm.yml", environment: "NPM_PUBLISH" } },
        }),
        { trustRepo: "oovm/npm-tools", trustFile: "publish-npm.yml", trustEnvironment: "NPM_PUBLISH" },
    );
    assert.deepEqual(trustPayloadFromConfig({}), {});
});

test("passes an explicit empty value through so the native layer can reject it", () => {
    assert.deepEqual(trustPayloadFromConfig({ trust: { npm: { environment: "" } } }), { trustEnvironment: "" });
});
