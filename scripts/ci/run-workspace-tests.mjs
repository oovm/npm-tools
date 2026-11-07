import { execFileSync, spawn } from "node:child_process";
import { readFileSync } from "node:fs";

/**
 * Run the workspace tests and assert that every member's test target actually produced a
 * `Running` line.
 *
 * A failing run and an incomplete run both look red, so failure alone cannot prove that the
 * remaining crates executed. Cargo's default fail-fast stopped the run at the first failing
 * target and left the rest unexecuted while the job still reported "failure".
 *
 * Usage: `node scripts/ci/run-workspace-tests.mjs [--log <path>]`
 * `--log` checks an existing cargo log instead of running the tests (used to prove the check).
 */
const CARGO_ARGS = ["test", "--release", "--workspace", "--no-fail-fast", "--locked"];

const logFlag = process.argv.indexOf("--log");
const logPath = logFlag === -1 ? null : process.argv[logFlag + 1];
if (logFlag !== -1 && !logPath) {
    console.error("`--log` requires a path.");
    process.exit(1);
}

function expectedTargets() {
    // `--no-deps` avoids resolving (and rewriting) the lock, which is checked separately.
    const raw = execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], { encoding: "utf8" });
    const metadata = JSON.parse(raw);
    const members = new Set(metadata.workspace_members);
    return metadata.packages
        .filter((pkg) => members.has(pkg.id))
        .filter((pkg) => pkg.targets.some((target) => target.test))
        .map((pkg) => pkg.name.replaceAll("-", "_"));
}

function runningLines(output) {
    // Strip real escape sequences and the caret-rendered form that survives `gh run view > file`.
    const plain = output.replace(/\u001b\[[0-9;]*m/g, "").replace(/\^\[\[[0-9;]*m/g, "");
    return plain.split(/\r?\n/).filter((line) => /Running (unittests|tests)/.test(line));
}

function runCargo() {
    return new Promise((resolve, reject) => {
        const child = spawn("cargo", CARGO_ARGS, { stdio: ["ignore", "pipe", "pipe"] });
        let output = "";
        child.stdout.on("data", (chunk) => {
            output += chunk;
            process.stdout.write(chunk);
        });
        child.stderr.on("data", (chunk) => {
            output += chunk;
            process.stderr.write(chunk);
        });
        child.on("error", reject);
        child.on("close", (code) => resolve({ code, output }));
    });
}

async function main() {
    const { code, output } = logPath ? { code: 0, output: readFileSync(logPath, "utf8") } : await runCargo();
    const expected = expectedTargets();
    const lines = runningLines(output);
    const missing = expected.filter((name) => !lines.some((line) => line.includes(name)));

    let failed = code !== 0;
    if (missing.length > 0) {
        failed = true;
        console.error(`These workspace test targets produced no "Running" line: ${missing.join(", ")}`);
        console.error("A crate that failed to compile, was feature-gated, or was renamed can leave the job");
        console.error("red while its tests never ran. Treat this as a test-coverage failure, not a test failure.");
    }

    if (!failed) {
        console.log(`All ${expected.length} workspace test targets ran.`);
    }
    process.exit(failed ? 1 : 0);
}

await main();
