/**
 * Publish gate: require green workflow runs on the tag-peeled commit.
 *
 * Contract (release gate, not a convenience check):
 * - A failed workflow never passes, and an earlier success on the same SHA must
 *   not mask a later completed failure (re-run, extra run, another event).
 * - Every required job must conclude `success` on the accepted run. A workflow
 *   can conclude `success` while one of its jobs was skipped, cancelled or timed
 *   out, so job-level evidence is checked instead of the workflow conclusion.
 * - Only runs whose `headSha` equals the bound tag SHA are considered.
 */

import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const REPO = process.env.GITHUB_REPOSITORY || "oovm/npm-tools";
const TAG_SHA = String(process.env.TAG_SHA || process.argv.find((arg) => arg.startsWith("--sha="))?.slice(6) || "")
    .trim()
    .toLowerCase();
const WORKFLOWS = (process.env.CI_WORKFLOW_FILES || process.env.CI_WORKFLOW_FILE || "check-rust.yml")
    .split(",")
    .map((workflow) => workflow.trim())
    .filter(Boolean);
const TIMEOUT_MS = Number(process.env.CI_WAIT_TIMEOUT_MS || 45 * 60 * 1000);
const INTERVAL_MS = Number(process.env.CI_WAIT_INTERVAL_MS || 20 * 1000);

// Job names as GitHub Actions reports them, verified against real runs. A matrix
// job is `build (<os>)` (no `name:` on the job), the TypeScript job is `pnpm packages`.
export const REQUIRED_JOBS_BY_WORKFLOW = {
    "check-rust.yml": ["build (ubuntu-latest)", "build (macos-latest)", "build (windows-latest)"],
    "check-typescript.yml": ["pnpm packages"],
};

const ACTIVE_STATUSES = new Set(["queued", "in_progress", "pending", "requested", "waiting"]);
const WAIT_STATES = new Set(["active", "no-runs", "no-completed"]);

/**
 * Decide one workflow gate from run metadata only (no gh calls), so fixtures can drive it.
 *
 * @param {object} input
 * @param {string} input.sha bound tag SHA (40-char hex, lower-case)
 * @param {string} input.workflow workflow file name
 * @param {Array<object>} input.runs `gh run list` entries, any order
 * @param {(runId: number) => Array<{name: string, conclusion: string|null, status?: string}>} [input.jobsByRunId]
 */
export function evaluateWorkflow({ sha, workflow, runs, jobsByRunId }) {
    if (!REQUIRED_JOBS_BY_WORKFLOW[workflow]) {
        return { ok: false, state: "unconfigured-workflow" };
    }

    const scoped = (runs ?? []).filter((run) => String(run?.headSha ?? "").toLowerCase() === sha);
    if (scoped.length === 0) {
        return { ok: false, state: "no-runs" };
    }

    const active = scoped.filter((run) => ACTIVE_STATUSES.has(String(run.status)));
    if (active.length > 0) {
        return { ok: false, state: "active", active: active.length };
    }

    const completed = scoped
        .filter((run) => String(run.status) === "completed")
        .sort((a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt) || (b.databaseId ?? 0) - (a.databaseId ?? 0));
    if (completed.length === 0) {
        return { ok: false, state: "no-completed" };
    }

    // The newest completed run on the bound SHA is the only run that can pass.
    const newest = completed[0];
    if (newest.conclusion !== "success") {
        return { ok: false, state: "run-not-success", runId: newest.databaseId, conclusion: newest.conclusion ?? null };
    }

    const jobs = jobsByRunId ? jobsByRunId(newest.databaseId) ?? [] : [];
    const byName = new Map(jobs.map((job) => [job.name, job]));
    for (const name of REQUIRED_JOBS_BY_WORKFLOW[workflow]) {
        const job = byName.get(name);
        if (!job) {
            return { ok: false, state: "missing-job", runId: newest.databaseId, job: name };
        }
        if (job.conclusion !== "success") {
            return {
                ok: false,
                state: "job-not-success",
                runId: newest.databaseId,
                job: name,
                conclusion: job.conclusion ?? job.status ?? null,
            };
        }
    }

    return { ok: true, state: "success", runId: newest.databaseId, jobCount: jobs.length };
}

export function describeFailure(result) {
    const where = `${result.workflow} on ${TAG_SHA}`;
    switch (result.state) {
        case "unconfigured-workflow":
            return `${result.workflow} has no required-job expectation in \`REQUIRED_JOBS_BY_WORKFLOW\``;
        case "run-not-success":
            return `newest completed run for ${where} is ${result.conclusion} (run ${result.runId})`;
        case "missing-job":
            return `required job "${result.job}" is missing from run ${result.runId} for ${where}`;
        case "job-not-success":
            return `required job "${result.job}" in run ${result.runId} for ${where} concluded ${result.conclusion} (missing/skipped/cancelled/timeout is not success)`;
        case "no-runs":
            return `no run found for ${where}`;
        case "no-completed":
            return `no completed run for ${where}`;
        default:
            return `${where} gate not satisfied (state=${result.state})`;
    }
}

function fail(msg) {
    console.error(`require-ci-success: ${msg}`);
    process.exit(1);
}

function gh(args) {
    const result = spawnSync("gh", args, {
        encoding: "utf8",
        shell: false,
        env: { ...process.env, GH_TOKEN: process.env.GH_TOKEN || process.env.GITHUB_TOKEN || "" },
    });
    if (result.status !== 0) {
        fail(result.stderr?.trim() || result.stdout?.trim() || `gh ${args.join(" ")} exit ${result.status}`);
    }
    return String(result.stdout ?? "").trim();
}

function listRuns(workflow) {
    const raw = gh([
        "run",
        "list",
        "--repo",
        REPO,
        "--commit",
        TAG_SHA,
        "--workflow",
        workflow,
        "--limit",
        "20",
        "--json",
        "databaseId,status,conclusion,createdAt,displayTitle,event,headSha",
    ]);
    try {
        return JSON.parse(raw);
    } catch {
        fail(`invalid gh json for ${workflow}: ${raw.slice(0, 200)}`);
    }
}

function jobsByRunId(runId) {
    const raw = gh(["api", `repos/${REPO}/actions/runs/${runId}/jobs?per_page=100`]);
    try {
        return (JSON.parse(raw).jobs ?? []).map((job) => ({ name: job.name, conclusion: job.conclusion, status: job.status }));
    } catch {
        fail(`invalid jobs json for run ${runId}: ${raw.slice(0, 200)}`);
    }
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
    if (!/^[0-9a-f]{40}$/.test(TAG_SHA)) {
        fail(`TAG_SHA must be 40-char hex (got '${TAG_SHA || "(empty)"}')`);
    }
    if (WORKFLOWS.length === 0) {
        fail("at least one CI workflow file is required");
    }

    const deadline = Date.now() + TIMEOUT_MS;
    let attempt = 0;

    for (;;) {
        attempt += 1;
        const results = WORKFLOWS.map((workflow) => ({
            workflow,
            ...evaluateWorkflow({ sha: TAG_SHA, workflow, runs: listRuns(workflow), jobsByRunId }),
        }));

        if (results.every((result) => result.ok)) {
            for (const result of results) {
                console.log(`CI success confirmed for ${result.workflow} on ${TAG_SHA} (run ${result.runId})`);
            }
            return;
        }

        const decided = results.filter((result) => !result.ok && !WAIT_STATES.has(result.state));
        if (decided.length > 0) {
            fail(decided.map(describeFailure).join(" | "));
        }

        if (Date.now() >= deadline) {
            const states = results.map((result) => `${result.workflow}:${result.state}`).join(", ");
            fail(`timed out after ${TIMEOUT_MS}ms waiting for green workflows (${states}) on ${TAG_SHA}`);
        }

        const waiting = results.map((result) => `${result.workflow}:${result.state}`).join(", ");
        console.log(`[${attempt}] ${waiting} on ${TAG_SHA.slice(0, 7)}…`);

        await sleep(INTERVAL_MS);
    }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
    main().catch((error) => fail(error instanceof Error ? error.message : String(error)));
}
