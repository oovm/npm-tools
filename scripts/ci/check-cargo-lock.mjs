import { readFileSync } from "node:fs";

/**
 * Reject a `Cargo.lock` that resolved `oak-*` from local paths.
 *
 * The local `.cargo/config.toml` patch rewrites those packages to path sources, which strips their
 * `source` line. Such a lock is valid only on a machine carrying the patch and can never satisfy
 * `cargo --locked` in CI, so it must never be committed.
 *
 * Usage: `node scripts/ci/check-cargo-lock.mjs [lockfile]`
 */
const file = process.argv[2] ?? "Cargo.lock";
const text = readFileSync(file, "utf8");

const entries = text.split(/\n\[\[package\]\]\n/).slice(1);
const pathSourced = [];
for (const entry of entries) {
    const name = entry.match(/^name = "(.+)"/m)?.[1];
    if (!name?.startsWith("oak-")) {
        continue;
    }
    if (!/^source = /m.test(entry)) {
        pathSourced.push(name);
    }
}

if (pathSourced.length > 0) {
    console.error(`Cargo.lock resolved oak packages from local paths: ${pathSourced.join(", ")}`);
    console.error("Regenerate the lock without the local path patch, then commit it.");
    process.exit(1);
}

console.log(`Cargo.lock source check passed (${file})`);
