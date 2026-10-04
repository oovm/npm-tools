/**
 * Publish gate: require green workflow runs on the tag commit.
 */

import { spawnSync } from "node:child_process";

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
        "databaseId,status,conclusion,createdAt,displayTitle,event",
    ]);
    try {
        return JSON.parse(raw);
    } catch {
        fail(`invalid gh json for ${workflow}: ${raw.slice(0, 200)}`);
    }
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

function workflowState(workflow) {
    const runs = listRuns(workflow);
    const active = runs.filter((run) => run.status === "queued" || run.status === "in_progress" || run.status === "pending");
    const successes = runs.filter((run) => run.status === "completed" && run.conclusion === "success");
    const failures = runs.filter(
        (run) => run.status === "completed" && run.conclusion && run.conclusion !== "success" && run.conclusion !== "skipped",
    );
    return { workflow, runs, active, successes, failures };
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

    while (Date.now() < deadline) {
        attempt += 1;
        const states = WORKFLOWS.map((workflow) => workflowState(workflow));
        const pending = states.filter((state) => state.successes.length === 0 || state.active.length > 0);
        const failed = states.filter((state) => state.successes.length === 0 && state.failures.length > 0 && state.active.length === 0);

        if (pending.length === 0) {
            for (const state of states) {
                const pick = state.successes[0];
                console.log(
                    `CI success confirmed for ${state.workflow} on ${TAG_SHA} (run ${pick.databaseId} "${pick.displayTitle}" event=${pick.event})`,
                );
            }
            return;
        }

        if (failed.length > 0) {
            const state = failed[0];
            const last = state.failures[0];
            fail(
                `CI on ${TAG_SHA} has no success for ${state.workflow} (${state.failures.length} completed non-success. latest ${last.databaseId} conclusion=${last.conclusion})`,
            );
        }

        const summary = states
            .map((state) => {
                if (state.runs.length === 0) {
                    return `${state.workflow}:waiting`;
                }
                if (state.active.length > 0) {
                    return `${state.workflow}:active(${state.active.length})`;
                }
                return `${state.workflow}:pending`;
            })
            .join(", ");
        console.log(`[${attempt}] ${summary} on ${TAG_SHA.slice(0, 7)}…`);

        await sleep(INTERVAL_MS);
    }

    fail(`timed out after ${TIMEOUT_MS}ms waiting for green workflows (${WORKFLOWS.join(", ")}) on ${TAG_SHA}`);
}

main().catch((error) => fail(error instanceof Error ? error.message : String(error)));
