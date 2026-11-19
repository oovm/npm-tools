import { readFileSync } from "node:fs";

/**
 * Reject a `Cargo.lock` that still carries legacy Oak/Acorn packages, or that resolved a Boost
 * package from a local path.
 *
 * The local `.cargo/config.toml` patch rewrites Boost packages to path sources, which strips
 * their `source` line. Such a lock is valid only on a machine carrying the patch and can never
 * satisfy `cargo --locked` in CI, so it must never be committed.
 *
 * Usage: `node scripts/ci/check-cargo-lock.mjs [lockfile]`
 */
const file = process.argv[2] ?? "Cargo.lock";
const text = readFileSync(file, "utf8");

const entries = text.split(/\n\[\[package\]\]\n/).slice(1);
const legacy = [];
const pathSourced = [];
for (const entry of entries) {
    const name = entry.match(/^name = "(.+)"/m)?.[1];
    if (!name) {
        continue;
    }
    if (name.startsWith("oak-") || name.startsWith("acorn-")) {
        legacy.push(name);
        continue;
    }
    const boostOwned = name.startsWith("boost") || name.startsWith("data-") || name.startsWith("language-");
    if (boostOwned && !/^source = /m.test(entry)) {
        pathSourced.push(name);
    }
}

if (legacy.length > 0) {
    console.error(`Cargo.lock still resolves legacy packages: ${legacy.join(", ")}`);
    console.error("Switch the consumer to the Boost packages and regenerate the lock.");
    process.exit(1);
}

if (pathSourced.length > 0) {
    console.error(`Cargo.lock resolved Boost packages from local paths: ${pathSourced.join(", ")}`);
    console.error("Regenerate the lock without the local path patch, then commit it.");
    process.exit(1);
}

console.log(`Cargo.lock source check passed (${file})`);
