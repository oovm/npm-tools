/**
 * Fixtures for the Nifty release gate. They prove that a red workflow, a missing
 * job, a skipped/cancelled job, an in-flight run, a later failed re-run or a run
 * on another SHA can never be accepted as publish evidence.
 */

import assert from "node:assert/strict";
import test from "node:test";

import { REQUIRED_JOBS_BY_WORKFLOW, evaluateWorkflow } from "./require-ci-success.mjs";

const RUST = "check-rust.yml";
const TS = "check-typescript.yml";
const SHA = "a".repeat(40);
const OTHER_SHA = "b".repeat(40);

function jobsFor(workflow) {
    return REQUIRED_JOBS_BY_WORKFLOW[workflow].map((name) => ({ name, conclusion: "success" }));
}

function run(overrides = {}) {
    return {
        databaseId: 1,
        status: "completed",
        conclusion: "success",
        createdAt: "2026-10-20T00:00:00Z",
        displayTitle: "CI",
        event: "push",
        headSha: SHA,
        ...overrides,
    };
}

test("same SHA, every required job green", () => {
    for (const workflow of [RUST, TS]) {
        const result = evaluateWorkflow({ sha: SHA, workflow, runs: [run()], jobsByRunId: () => jobsFor(workflow) });
        assert.equal(result.ok, true, workflow);
        assert.equal(result.runId, 1);
    }
});

test("failed workflow never passes", () => {
    const result = evaluateWorkflow({
        sha: SHA,
        workflow: RUST,
        runs: [run({ conclusion: "failure" })],
        jobsByRunId: () => jobsFor(RUST),
    });
    assert.equal(result.ok, false);
    assert.equal(result.state, "run-not-success");
    assert.equal(result.conclusion, "failure");
});

test("a required matrix job missing from the run", () => {
    const jobs = jobsFor(RUST).filter((job) => job.name !== "build (macos-latest)");
    const result = evaluateWorkflow({ sha: SHA, workflow: RUST, runs: [run()], jobsByRunId: () => jobs });
    assert.equal(result.ok, false);
    assert.equal(result.state, "missing-job");
    assert.equal(result.job, "build (macos-latest)");
});

test("skipped job is not success even when the workflow concluded success", () => {
    const jobs = jobsFor(RUST).map((job) =>
        job.name === "build (windows-latest)" ? { name: job.name, conclusion: "skipped" } : job,
    );
    const result = evaluateWorkflow({ sha: SHA, workflow: RUST, runs: [run()], jobsByRunId: () => jobs });
    assert.equal(result.ok, false);
    assert.equal(result.state, "job-not-success");
    assert.equal(result.job, "build (windows-latest)");
    assert.equal(result.conclusion, "skipped");
});

test("cancelled job is not success", () => {
    const jobs = jobsFor(TS).map((job) => ({ name: job.name, conclusion: "cancelled" }));
    const result = evaluateWorkflow({ sha: SHA, workflow: TS, runs: [run()], jobsByRunId: () => jobs });
    assert.equal(result.ok, false);
    assert.equal(result.state, "job-not-success");
});

test("in-flight run keeps the gate waiting", () => {
    const result = evaluateWorkflow({
        sha: SHA,
        workflow: TS,
        runs: [run({ databaseId: 2, status: "in_progress", conclusion: null, createdAt: "2026-10-20T01:00:00Z" }), run()],
        jobsByRunId: () => jobsFor(TS),
    });
    assert.equal(result.ok, false);
    assert.equal(result.state, "active");
});

test("a later failed re-run is not masked by an earlier success", () => {
    const result = evaluateWorkflow({
        sha: SHA,
        workflow: RUST,
        runs: [
            run({ databaseId: 9, conclusion: "failure", createdAt: "2026-10-20T02:00:00Z" }),
            run({ databaseId: 8, conclusion: "success", createdAt: "2026-10-20T01:00:00Z" }),
        ],
        jobsByRunId: () => jobsFor(RUST),
    });
    assert.equal(result.ok, false);
    assert.equal(result.state, "run-not-success");
    assert.equal(result.runId, 9);
});

test("a later successful re-run clears an earlier failure", () => {
    const result = evaluateWorkflow({
        sha: SHA,
        workflow: RUST,
        runs: [
            run({ databaseId: 11, conclusion: "success", createdAt: "2026-10-20T03:00:00Z" }),
            run({ databaseId: 10, conclusion: "failure", createdAt: "2026-10-20T02:00:00Z" }),
        ],
        jobsByRunId: () => jobsFor(RUST),
    });
    assert.equal(result.ok, true);
    assert.equal(result.runId, 11);
});

test("runs on a different SHA are ignored", () => {
    const result = evaluateWorkflow({
        sha: SHA,
        workflow: RUST,
        runs: [run({ headSha: OTHER_SHA })],
        jobsByRunId: () => jobsFor(RUST),
    });
    assert.equal(result.ok, false);
    assert.equal(result.state, "no-runs");
});

test("no runs yet", () => {
    const result = evaluateWorkflow({ sha: SHA, workflow: RUST, runs: [], jobsByRunId: () => jobsFor(RUST) });
    assert.equal(result.ok, false);
    assert.equal(result.state, "no-runs");
});

test("a workflow without a required-job expectation cannot pass", () => {
    assert.deepEqual(REQUIRED_JOBS_BY_WORKFLOW["publish-npm.yml"], undefined);
    const result = evaluateWorkflow({
        sha: SHA,
        workflow: "publish-npm.yml",
        runs: [run()],
        jobsByRunId: () => jobsFor(RUST),
    });
    assert.equal(result.ok, false);
    assert.equal(result.state, "unconfigured-workflow");
});
