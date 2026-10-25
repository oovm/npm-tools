import assert from "node:assert/strict";
import test from "node:test";

import { REQUIRED_WORKFLOWS, evaluateGate } from "./require-ci-success.mjs";

const SHA = "b".repeat(40);
const RUST = ".github/workflows/check-rust.yml";
const TYPESCRIPT = ".github/workflows/check-typescript.yml";

function run(id, path, overrides = {}) {
    return {
        id,
        path,
        head_sha: SHA,
        status: "completed",
        conclusion: "success",
        event: "push",
        head_branch: "dev",
        created_at: `2026-10-25T00:0${id}:00Z`,
        html_url: `https://example.test/runs/${id}`,
        ...overrides,
    };
}

function jobs(names, conclusion = "success") {
    return names.map((name) => ({ name, conclusion, status: "completed" }));
}

function decide(runs, jobsByRunId) {
    return evaluateGate({ sha: SHA, runs, jobsByRunId: (runId) => jobsByRunId[runId] ?? [] });
}

const rustWorkflow = REQUIRED_WORKFLOWS.find((workflow) => workflow.path === RUST);
const typescriptWorkflow = REQUIRED_WORKFLOWS.find((workflow) => workflow.path === TYPESCRIPT);

function greenJobs() {
    return {
        10: jobs(rustWorkflow.jobs),
        20: jobs(typescriptWorkflow.jobs),
    };
}

function greenRuns() {
    return [run(10, RUST), run(20, TYPESCRIPT)];
}

test("accepts a fully green CI for the bound SHA", () => {
    const result = decide(greenRuns(), greenJobs());

    assert.equal(result.ok, true);
    assert.equal(result.state, "success");
    assert.deepEqual(
        result.evidence.map((entry) => entry.workflow),
        [RUST, TYPESCRIPT],
    );
});

test("rejects a required workflow that completed without the required job", () => {
    const jobsByRunId = greenJobs();
    jobsByRunId[10] = jobs(rustWorkflow.jobs.filter((name) => name !== "Dependency policy"));

    const result = decide(greenRuns(), jobsByRunId);

    assert.equal(result.ok, false);
    assert.equal(result.state, "missing-job");
    assert.equal(result.job, "Dependency policy");
});

test("rejects a skipped required job even when the workflow concluded success", () => {
    const jobsByRunId = greenJobs();
    jobsByRunId[10] = jobs(rustWorkflow.jobs).map((job) =>
        job.name === "build (windows-latest)" ? { ...job, conclusion: "skipped" } : job,
    );

    const result = decide(greenRuns(), jobsByRunId);

    assert.equal(result.ok, false);
    assert.equal(result.state, "job-not-success");
    assert.equal(result.job, "build (windows-latest)");
    assert.equal(result.conclusion, "skipped");
});

test("rejects a cancelled or timed-out job", () => {
    for (const conclusion of ["cancelled", "timed_out"]) {
        const jobsByRunId = greenJobs();
        jobsByRunId[20] = jobs(typescriptWorkflow.jobs).map((job) => ({ ...job, conclusion }));

        const result = decide(greenRuns(), jobsByRunId);

        assert.equal(result.ok, false, conclusion);
        assert.equal(result.state, "job-not-success", conclusion);
        assert.equal(result.conclusion, conclusion);
    }
});

test("a newer failed run overrides an older green run on the same SHA", () => {
    const runs = [...greenRuns(), run(21, TYPESCRIPT, { conclusion: "failure", created_at: "2026-10-25T01:00:00Z" })];
    const result = decide(runs, greenJobs());

    assert.equal(result.ok, false);
    assert.equal(result.state, "run-not-success");
    assert.equal(result.conclusion, "failure");
});

test("waits for an in-progress run instead of passing or failing early", () => {
    const runs = [...greenRuns(), run(22, RUST, { status: "in_progress", conclusion: null })];
    const result = decide(runs, greenJobs());

    assert.equal(result.ok, false);
    assert.equal(result.state, "active");
    assert.equal(result.workflow, RUST);
});

test("does not accept a pull_request run as evidence for the branch head", () => {
    const runs = greenRuns().map((entry) => ({ ...entry, event: "pull_request" }));
    const result = decide(runs, greenJobs());

    assert.equal(result.ok, false);
    assert.equal(result.state, "no-runs");
});
