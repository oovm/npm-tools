/**
 * Rust dependency policy gate.
 *
 * Forbidden families: `notify` and its adapters, the `tracing` family,
 * `reqwest`/`ureq`, `walkdir` and `tempfile`. A workspace member must never
 * declare them, and no crate we own may pull them in. Reintroducing an edge is
 * a failure no matter how it is hidden: the graph is resolved with
 * `--all-features` for every platform target, so an optional feature or a
 * platform-only dependency cannot excuse it.
 *
 * Third-party frameworks we consume but do not fork (`gix`, `criterion`,
 * `axum`, `tauri`, `native-tls`) carry their own transitive dependencies. Those
 * edges stay out of the migration scope, so traversal stops at a registered
 * root and the packages behind it are reported for audit. A path that avoids
 * every registered root is a violation.
 *
 * Usage: node scripts/ci/check-rust-dependency-policy.mjs
 */

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const FORBIDDEN_FAMILIES = [
    {
        family: "notify",
        matches: (name) => name === "notify" || name.startsWith("notify-"),
        reason: "`fs-tools` owns watch events",
    },
    {
        family: "tracing",
        matches: (name) => name === "tracing" || name.startsWith("tracing-"),
        reason: "`logger` owns structured events and filters",
    },
    {
        family: "http-client",
        matches: (name) => name === "reqwest" || name === "ureq",
        reason: "`request` owns HTTP transport",
    },
    {
        family: "walkdir",
        matches: (name) => name === "walkdir",
        reason: "`fs-tools::walk` owns traversal and pruning",
    },
    {
        family: "tempfile",
        matches: (name) => name === "tempfile",
        reason: "`fs-tools::temp` owns temporary resources",
    },
];

// Frameworks whose own dependency graph is out of scope. Each entry documents
// why it is registered so a future removal has a closable condition.
const REGISTERED_THIRD_PARTY_ROOTS = [
    { name: "gix", condition: "removed when `nifty-git`/`nifty-history` stop consuming `gix`" },
    { name: "criterion", condition: "benchmark framework, replaced by an in-house harness or removed" },
    { name: "axum", condition: "server framework in upstream products, not owned by this repository" },
    { name: "tauri", condition: "desktop shell, not owned by this repository" },
    { name: "native-tls", condition: "TLS backend under an independent transport contract" },
];

function fail(msg) {
    console.error(`check-rust-dependency-policy: ${msg}`);
    process.exit(1);
}

function metadata() {
    const result = spawnSync("cargo", ["metadata", "--format-version", "1", "--all-features"], {
        cwd: ROOT,
        encoding: "utf8",
        maxBuffer: 128 * 1024 * 1024,
        shell: false,
    });
    if (result.status !== 0) {
        fail(`cargo metadata failed: ${result.stderr?.trim() || result.stdout?.trim() || result.status}`);
    }
    try {
        return JSON.parse(result.stdout);
    } catch {
        fail("cargo metadata returned invalid JSON");
    }
}

function forbiddenFamily(name) {
    return FORBIDDEN_FAMILIES.find((family) => family.matches(name)) ?? null;
}

function buildIndex(meta) {
    const names = new Map(meta.packages.map((pkg) => [pkg.id, pkg.name]));
    const edges = new Map();
    for (const node of meta.resolve?.nodes ?? []) {
        edges.set(node.id, (node.deps ?? []).map((dep) => dep.pkg));
    }
    return { names, edges };
}

/**
 * Collect every package reachable from the workspace members without passing a
 * registered third-party root. Registered roots are terminal for this walk, so
 * everything behind them is not considered a violation.
 */
function reachableWithoutRoots(start, edges, names, roots) {
    const parent = new Map();
    const queue = [...start];
    const seen = new Set(queue);
    for (let i = 0; i < queue.length; i += 1) {
        const id = queue[i];
        for (const next of edges.get(id) ?? []) {
            if (seen.has(next)) continue;
            seen.add(next);
            if (roots.has(names.get(next))) continue;
            parent.set(next, id);
            queue.push(next);
        }
    }
    return { packageIds: queue, parent };
}

/** Shortest dependency chain from any workspace member to `target`. */
function shortestPath(start, edges, target) {
    const parent = new Map(start.map((id) => [id, null]));
    const queue = [...start];
    for (let i = 0; i < queue.length; i += 1) {
        const id = queue[i];
        if (id === target) break;
        for (const next of edges.get(id) ?? []) {
            if (parent.has(next)) continue;
            parent.set(next, id);
            queue.push(next);
        }
    }
    if (!parent.has(target)) return null;
    const chain = [];
    for (let id = target; id !== null; id = parent.get(id)) chain.unshift(id);
    return chain;
}

function chainNames(chain, names) {
    return chain.map((id) => names.get(id));
}

function main() {
    const meta = metadata();
    if (!meta.resolve) fail("cargo metadata has no resolve graph");

    const { names, edges } = buildIndex(meta);
    const members = meta.workspace_members;
    const roots = new Set(REGISTERED_THIRD_PARTY_ROOTS.map((root) => root.name));
    const restricted = reachableWithoutRoots(members, edges, names, roots);

    const violations = [];
    for (const id of restricted.packageIds) {
        const name = names.get(id);
        const family = forbiddenFamily(name);
        if (!family) continue;
        const viaRestricted = [];
        for (let cur = id; cur !== undefined; cur = restricted.parent.get(cur)) {
            viaRestricted.unshift(names.get(cur));
            if (!restricted.parent.has(cur)) break;
        }
        const chain = viaRestricted.length > 1 ? viaRestricted : chainNames(shortestPath(members, edges, id) ?? [id], names);
        violations.push({ name, family: family.family, reason: family.reason, chain });
    }

    // Forbidden packages that only appear behind a registered root are listed for
    // audit, with the framework that owns them.
    const restrictedIds = new Set(restricted.packageIds);
    const allowed = [];
    for (const node of meta.resolve.nodes) {
        const name = names.get(node.id);
        const family = forbiddenFamily(name);
        if (!family || restrictedIds.has(node.id)) continue;
        const chain = shortestPath(members, edges, node.id);
        const owner = (chain ?? []).map((id) => names.get(id)).find((entry) => roots.has(entry)) ?? "(unknown)";
        allowed.push({ name, family: family.family, owner });
    }

    allowed.sort((a, b) => a.name.localeCompare(b.name));
    for (const entry of allowed) {
        console.log(`registered third-party transitive: ${entry.name} (${entry.family}) behind ${entry.owner}`);
    }

    if (violations.length > 0) {
        for (const violation of violations) {
            console.error(
                `forbidden dependency ${violation.name} (${violation.family}) is reachable without a registered third-party root`,
            );
            console.error(`  introduced by: ${violation.chain.join(" -> ")}`);
            console.error(`  owner of the capability: ${violation.reason}`);
        }
        fail(
            `${violations.length} forbidden dependency path(s); drop the edge instead of allowlisting it or disabling a capability`,
        );
    }

    const rootList = REGISTERED_THIRD_PARTY_ROOTS.map((root) => root.name).join(", ");
    console.log(
        `dependency policy ok: ${members.length} workspace members, ${meta.resolve.nodes.length} resolved packages, registered third-party roots: ${rootList}`,
    );
}

main();
