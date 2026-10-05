import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import ts from "typescript";

test("reports only the unknown config key and contextually types config factories", () => {
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
    assert.equal(diagnostics[0].code, 2353);
    assert.match(ts.flattenDiagnosticMessageText(diagnostics[0].messageText, " "), /unknownOption.*does not exist in type/);
});
