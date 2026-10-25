/**
 * Publish gate: require green CI on the exact tag-peeled commit.
 *
 * Contract (release gate, not a convenience check):
 * - Every required workflow must have a completed `push` run whose `head_sha`
 *   equals the bound SHA. `pull_request` runs never stand in for it.
 * - The newest completed run on that SHA decides. A failed workflow never
 *   passes, and an older success on the same SHA must not mask a later failure.
 * - Every required job must exist and conclude `success`. Missing, skipped,
 *   cancelled, timed-out and failing jobs are not success.
 * - Queued or running workflows are waited for until `CI_WAIT_TIMEOUT_MS`.
 *   Waiting is not a bypass: a decided failure exits immediately.
 *
 * Usage (CI):
 *   GITHUB_TOKEN=… GITHUB_REPOSITORY=oovm/npm-tools \
 *     node scripts/ci/require-ci-success.mjs --sha=<tag-peeled sha>
 */

import { fileURLToPath } from "node:url";

const API = "https://api.github.com";
const RUN_PAGE = 100;
const JOB_PAGE = 100;
const REQUEST_TIMEOUT_MS = 30 * 1000;

// Job names are the ones GitHub reports for the checked-in workflows. Matrix jobs
// keep the `job (os)` shape, so a shrunken or renamed matrix fails the gate.
export const REQUIRED_WORKFLOWS = [
    {
        path: ".github/workflows/check-rust.yml",
        jobs: ["build (ubuntu-latest)", "build (macos-latest)", "build (windows-latest)", "Dependency policy"],
    },
    {
        path: ".github/workflows/check-typescript.yml",
        jobs: ["pnpm packages"],
    },
];

const ACTIVE_STATUSES = new Set(["queued", "in_progress", "pending", "requested", "waiting"]);

function scopeRuns({ sha, workflow, runs }) {
    // Only branch pushes count. A `pull_request` run can share the same SHA and
    // must not stand in for (or veto) the run that validates the branch head.
    const scoped = (runs ?? []).filter(
        (run) =>
            String(run?.head_sha ?? "").toLowerCase() === sha &&
            run?.path === workflow.path &&
            run?.event === "push",
    );
    const active = scoped.filter((run) => ACTIVE_STATUSES.has(String(run.status)));
    const completed = scoped
        .filter((run) => String(run.status) === "completed")
        .sort((a, b) => Date.parse(b.created_at) - Date.parse(a.created_at) || b.id - a.id);
    return { scoped, active, completed };
}

/** Newest completed run for one required workflow, or `null`. */
export function newestCompletedRun({ sha, workflow, runs }) {
    return scopeRuns({ sha, workflow, runs }).completed[0] ?? null;
}

/**
 * Decide one required workflow from run/job metadata only (no I/O), so fixtures
 * can drive it.
 *
 * @param {object} input
 * @param {string} input.sha bound tag SHA (40-char hex, lower-case)
 * @param {object} input.workflow entry of `REQUIRED_WORKFLOWS`
 * @param {Array<object>} input.runs GitHub workflow runs for the repository
 * @param {(runId: number) => Array<{name: string, conclusion: string|null, status?: string}>} [input.jobsByRunId]
 */
export function evaluateWorkflow({ sha, workflow, runs, jobsByRunId }) {
    const { scoped, active, completed } = scopeRuns({ sha, workflow, runs });
    if (scoped.length === 0) {
        return { ok: false, state: "no-runs", workflow: workflow.path };
    }
    if (active.length > 0) {
        return { ok: false, state: "active", workflow: workflow.path };
    }
    if (completed.length === 0) {
        return { ok: false, state: "no-completed", workflow: workflow.path };
    }

    const newest = completed[0];
    if (newest.conclusion !== "success") {
        return {
            ok: false,
            state: "run-not-success",
            workflow: workflow.path,
            runId: newest.id,
            conclusion: newest.conclusion ?? null,
            url: newest.html_url,
        };
    }

    const jobs = jobsByRunId ? jobsByRunId(newest.id) ?? [] : [];
    const byName = new Map(jobs.map((job) => [job.name, job]));
    for (const name of workflow.jobs) {
        const job = byName.get(name);
        if (!job) {
            return { ok: false, state: "missing-job", workflow: workflow.path, runId: newest.id, job: name, url: newest.html_url };
        }
        if (job.conclusion !== "success") {
            return {
                ok: false,
                state: "job-not-success",
                workflow: workflow.path,
                runId: newest.id,
                job: name,
                conclusion: job.conclusion ?? job.status ?? null,
                url: newest.html_url,
            };
        }
    }

    // Skipped or cancelled jobs elsewhere in a required workflow would otherwise
    // hide behind the workflow-level `success` conclusion.
    const notSuccess = jobs.find((job) => job.conclusion !== "success");
    if (notSuccess) {
        return {
            ok: false,
            state: "job-not-success",
            workflow: workflow.path,
            runId: newest.id,
            job: notSuccess.name,
            conclusion: notSuccess.conclusion ?? notSuccess.status ?? null,
            url: newest.html_url,
        };
    }

    return {
        ok: true,
        state: "success",
        workflow: workflow.path,
        runId: newest.id,
        url: newest.html_url,
        event: newest.event,
        branch: newest.head_branch,
    };
}

/**
 * Decide the whole gate: every required workflow must be green.
 *
 * @param {object} input
 * @param {string} input.sha bound tag SHA
 * @param {Array<object>} input.runs workflow runs for the repository
 * @param {(runId: number) => Array<object>} [input.jobsByRunId]
 */
export function evaluateGate({ sha, runs, jobsByRunId }) {
    const evidence = [];
    for (const workflow of REQUIRED_WORKFLOWS) {
        const result = evaluateWorkflow({ sha, workflow, runs, jobsByRunId });
        if (!result.ok) return result;
        evidence.push({ workflow: result.workflow, runId: result.runId, url: result.url, event: result.event });
    }
    return { ok: true, state: "success", evidence };
}

export function describeFailure(result) {
    const workflow = result.workflow ? ` ${result.workflow}` : "";
    switch (result.state) {
        case "run-not-success":
            return `newest completed${workflow} run on the bound SHA is ${result.conclusion} (run ${result.runId}); a failed workflow never satisfies the release gate`;
        case "missing-job":
            return `required job "${result.job}" is missing from${workflow} run ${result.runId}`;
        case "job-not-success":
            return `required job "${result.job}" in${workflow} run ${result.runId} concluded ${result.conclusion} (missing, skipped, cancelled and timed out are not success)`;
        case "no-runs":
            return `no push${workflow} run found for the bound SHA`;
        case "no-completed":
            return `no completed push${workflow} run found for the bound SHA`;
        default:
            return `gate not satisfied (state=${result.state})`;
    }
}

function fail(msg) {
    console.error(`require-ci-success: ${msg}`);
    process.exit(1);
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

function apiHeaders(token) {
    return {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${token}`,
        "User-Agent": "nifty-publish-gate",
        "X-GitHub-Api-Version": "2022-11-28",
    };
}

async function requestJson(token, url) {
    const response = await fetch(url, {
        headers: apiHeaders(token),
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
    const body = await response.text();
    if (!response.ok) {
        fail(`GitHub API ${response.status} for ${url}: ${body.slice(0, 300)}`);
    }
    try {
        return JSON.parse(body);
    } catch {
        fail(`invalid GitHub API JSON for ${url}: ${body.slice(0, 200)}`);
    }
}

async function listRuns(token, repo, sha) {
    const data = await requestJson(
        token,
        `${API}/repos/${repo}/actions/runs?head_sha=${sha}&per_page=${RUN_PAGE}`,
    );
    return data.workflow_runs ?? [];
}

async function listJobs(token, repo, runId) {
    const jobs = [];
    for (let page = 1; ; page += 1) {
        const data = await requestJson(
            token,
            `${API}/repos/${repo}/actions/runs/${runId}/jobs?per_page=${JOB_PAGE}&page=${page}`,
        );
        const batch = data.jobs ?? [];
        jobs.push(...batch.map((job) => ({ name: job.name, conclusion: job.conclusion, status: job.status })));
        if (batch.length < JOB_PAGE) return jobs;
    }
}

/** Fetch the job list for every run the decision may consult. */
async function jobsForDecision(token, repo, sha, runs) {
    const jobsByRunId = new Map();
    for (const workflow of REQUIRED_WORKFLOWS) {
        const newest = newestCompletedRun({ sha, workflow, runs });
        if (newest) jobsByRunId.set(newest.id, await listJobs(token, repo, newest.id));
    }
    return (runId) => jobsByRunId.get(runId) ?? [];
}

async function main() {
    const sha = String(process.env.CI_BOUND_SHA || process.argv.find((arg) => arg.startsWith("--sha="))?.slice(6) || "")
        .trim()
        .toLowerCase();
    if (!/^[0-9a-f]{40}$/.test(sha)) {
        fail(`bound SHA must be 40-char hex (got '${sha || "(empty)"}')`);
    }

    const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN || "";
    if (!token) fail("GITHUB_TOKEN is required to read CI runs");
    const repo = process.env.GITHUB_REPOSITORY || "";
    if (!/^[^/]+\/[^/]+$/.test(repo)) {
        fail(`GITHUB_REPOSITORY must be owner/name (got '${repo || "(empty)"}')`);
    }

    const timeoutMs = Number(process.env.CI_WAIT_TIMEOUT_MS || 60 * 60 * 1000);
    const intervalMs = Number(process.env.CI_WAIT_INTERVAL_MS || 30 * 1000);
    const deadline = Date.now() + timeoutMs;

    for (let attempt = 1; ; attempt += 1) {
        const runs = await listRuns(token, repo, sha);
        const jobsByRunId = await jobsForDecision(token, repo, sha, runs);
        const result = evaluateGate({ sha, runs, jobsByRunId });

        if (result.ok) {
            for (const entry of result.evidence) {
                console.log(`CI success confirmed for ${sha}: ${entry.workflow} run ${entry.runId} (${entry.url})`);
            }
            return;
        }

        if (result.state === "active" || result.state === "no-runs" || result.state === "no-completed") {
            if (Date.now() >= deadline) {
                fail(
                    `timed out after ${timeoutMs}ms waiting for green CI on ${sha} (state=${result.state}, workflow=${result.workflow})`,
                );
            }
            console.log(`[${attempt}] ${describeFailure(result)} — waiting…`);
            await sleep(intervalMs);
            continue;
        }

        fail(describeFailure(result));
    }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
    main().catch((error) => fail(error instanceof Error ? error.message : String(error)));
}
