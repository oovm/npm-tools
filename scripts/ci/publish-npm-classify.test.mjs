import assert from "node:assert/strict";
import test from "node:test";

import { isAlreadyPublished, isAuthFailure } from "./publish-npm.mjs";

const STAGED_409 =
    'npm error code E409\nnpm error 409 Conflict - PUT https://registry.npmjs.org/@scope%2fpkg - Cannot publish over previously staged version "0.0.2".';

test("treats a staged 409 as an idempotent already-published outcome", () => {
    assert.equal(isAlreadyPublished(STAGED_409), true);
    assert.equal(isAuthFailure(STAGED_409), false);
});

test("keeps genuine auth failures out of the already-published class", () => {
    const oidc = "npm error code E401 unable to authenticate with OIDC trusted publisher";
    assert.equal(isAlreadyPublished(oidc), false);
    assert.equal(isAuthFailure(oidc), true);
});
