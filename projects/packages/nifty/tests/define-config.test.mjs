import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import ts from "typescript";

test("reports one direct object-shape error and contextually types config factories", () => {
    const fixture = path.resolve("tests/fixtures/define-config-invalid.ts");
    const options = {
        module: ts.ModuleKind.NodeNext,
        moduleResolution: ts.ModuleResolutionKind.NodeNext,
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: ts.ScriptTarget.ES2022,
    };
    const program = ts.createProgram([fixture], options);
    const diagnostics = ts.getPreEmitDiagnostics(program);

    assert.equal(diagnostics.length, 1);
    assert.equal(diagnostics[0].code, 2322);
    assert.match(ts.flattenDiagnosticMessageText(diagnostics[0].messageText, " "), /not assignable to type 'never'/);
    assert.equal(ts.getLineAndCharacterOfPosition(diagnostics[0].file, diagnostics[0].start).line + 1, 5);
});

test("preserves the exact return type for object and factory configs", () => {
    const fixture = path.resolve("tests/fixtures/define-config-valid.ts");
    const options = {
        module: ts.ModuleKind.NodeNext,
        moduleResolution: ts.ModuleResolutionKind.NodeNext,
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: ts.ScriptTarget.ES2022,
    };
    const program = ts.createProgram([fixture], options);
    assert.equal(ts.getPreEmitDiagnostics(program).length, 0);
});
