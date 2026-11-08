import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";
import ts from "typescript";

const OPTIONS = {
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    noEmit: true,
    skipLibCheck: true,
    strict: true,
    target: ts.ScriptTarget.ES2022,
};

function fixtureDiagnostics(name) {
    const program = ts.createProgram([path.resolve("tests/fixtures", name)], OPTIONS);
    return ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
        code: diagnostic.code,
        line: diagnostic.file && diagnostic.start != null ? ts.getLineAndCharacterOfPosition(diagnostic.file, diagnostic.start).line + 1 : null,
        message: ts.flattenDiagnosticMessageText(diagnostic.messageText, " "),
    }));
}

test("names the offending key and call form instead of collapsing to never", () => {
    const diagnostics = fixtureDiagnostics("define-config-invalid.ts");

    assert.deepEqual(
        diagnostics.map((diagnostic) => diagnostic.code),
        [2353, 2345],
    );
    assert.match(diagnostics[0].message, /'unknownOption' does not exist in type 'NiftyConfigExport'/);
    assert.equal(diagnostics[0].line, 5);
    assert.match(diagnostics[1].message, /not assignable to parameter of type 'NiftyConfigExport'/);
    assert.equal(diagnostics[1].line, 8);
});

test("accepts object and factory configs with contextual typing", () => {
    assert.deepEqual(fixtureDiagnostics("define-config-valid.ts"), []);
});
