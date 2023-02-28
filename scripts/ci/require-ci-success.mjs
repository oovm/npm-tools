/**
 * Publish gate: require a green workflow run on the tag commit.
 */

import { spawnSync } from "node:child_process";

const REPO = process.env.GITHUB_REPOSITORY || "oovm/npm-tools";
const TAG_SHA = String(process.env.TAG_SHA || process.argv.find((arg) => arg.startsWith("--sha="))?.slice(6) || "")
    .trim()
    .toLowerCase();
const WORKFLOW = process.env.CI_WORKFLOW_FILE || "rust.yml";
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

function listRuns() {
    const raw = gh([
        "run",
        "list",
        "--repo",
        REPO,
        "--commit",
        TAG_SHA,
        "--workflow",
        WORKFLOW,
        "--limit",
        "20",
        "--json",
        "databaseId,status,conclusion,createdAt,displayTitle,event",
    ]);
    try {
        return JSON.parse(raw);
    } catch {
        fail(`invalid gh json: ${raw.slice(0, 200)}`);
    }
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

async function main() {
    if (!/^[0-9a-f]{40}$/.test(TAG_SHA)) {
        fail(`TAG_SHA must be 40-char hex (got '${TAG_SHA || "(empty)"}')`);
    }

    const deadline = Date.now() + TIMEOUT_MS;
    let attempt = 0;

    while (Date.now() < deadline) {
        attempt += 1;
        const runs = listRuns();
        const active = runs.filter((run) => run.status === "queued" || run.status === "in_progress" || run.status === "pending");
        const successes = runs.filter((run) => run.status === "completed" && run.conclusion === "success");
        const failures = runs.filter(
            (run) => run.status === "completed" && run.conclusion && run.conclusion !== "success" && run.conclusion !== "skipped",
        );

        if (successes.length > 0 && active.length === 0) {
            const pick = successes[0];
            console.log(`CI success confirmed for ${TAG_SHA} (run ${pick.databaseId} "${pick.displayTitle}" event=${pick.event})`);
            return;
        }

        if (runs.length === 0) {
            console.log(`[${attempt}] waiting for first ${WORKFLOW} run on ${TAG_SHA.slice(0, 7)}…`);
        } else if (active.length > 0) {
            console.log(`[${attempt}] ${active.length} CI run(s) still active on ${TAG_SHA.slice(0, 7)}…`);
        } else if (successes.length === 0 && failures.length > 0) {
            const last = failures[0];
            fail(
                `CI on ${TAG_SHA} has no success (${failures.length} completed non-success. latest ${last.databaseId} conclusion=${last.conclusion})`,
            );
        } else {
            console.log(`[${attempt}] CI runs present but no success yet on ${TAG_SHA.slice(0, 7)}…`);
        }

        await sleep(INTERVAL_MS);
    }

    fail(`timed out after ${TIMEOUT_MS}ms waiting for green ${WORKFLOW} on ${TAG_SHA}`);
}

main().catch((error) => fail(error instanceof Error ? error.message : String(error)));
