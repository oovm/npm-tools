/**
 * GitHub Actions: publish npm-tools workspace packages (OIDC Trusted Publisher).
 *
 * - tag vX.Y.Z → version X.Y.Z
 * - idempotent: skip when version already on registry
 * - no NPM_TOKEN; OIDC only (permissions.id-token: write)
 * - contract: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/npm-tools
 *
 * Native artifacts: publish-npm.yml matrix → NIFTY_NATIVE_ARTIFACTS (dist/native-flat).
 */

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const REPO_URL = "git+https://github.com/oovm/npm-tools.git";

/** Matrix platforms → npm optional packages (@doki-land/nifty-<short>). */
const NATIVE_PLATFORMS = [
    {
        short: "win32-x64",
        nodeFile: "nifty-win32-x64-msvc.node",
        os: ["win32"],
        cpu: ["x64"],
        sourceDir: "projects/packages/nifty-win32-x64",
    },
    {
        short: "linux-x64",
        nodeFile: "nifty-linux-x64-gnu.node",
        os: ["linux"],
        cpu: ["x64"],
        sourceDir: "projects/packages/nifty-linux-x64",
    },
    {
        short: "darwin-arm64",
        nodeFile: "nifty-darwin-arm64.node",
        os: ["darwin"],
        cpu: ["arm64"],
        sourceDir: "projects/packages/nifty-darwin-arm64",
    },
];

const JS_PACKAGES = [
    { dir: "projects/packages/nifty", publishName: "@doki-land/nifty" },
    { dir: "projects/packages/nifty-skills", publishName: "@doki-land/nifty-skills" },
];

function fail(msg) {
    console.error(`ci-publish-npm: ${msg}`);
    process.exit(1);
}

function run(cmd, args, opts = {}) {
    const r = spawnSync(cmd, args, {
        cwd: opts.cwd ?? ROOT,
        encoding: "utf8",
        shell: process.platform === "win32",
        env: opts.env ?? process.env,
        stdio: opts.stdio ?? "pipe",
    });
    return {
        status: r.status ?? 1,
        stdout: String(r.stdout ?? "").trim(),
        stderr: String(r.stderr ?? "").trim(),
    };
}

function resolveVersion() {
    const fromArg = process.argv.find((a) => a.startsWith("--version="))?.slice("--version=".length);
    if (fromArg) return fromArg.replace(/^v/, "");
    const ref = process.env.GITHUB_REF ?? "";
    const m = ref.match(/^refs\/tags\/v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)$/);
    if (m) return m[1];
    fail("need --version=X.Y.Z or GITHUB_REF=refs/tags/vX.Y.Z");
}

function readJson(p) {
    return JSON.parse(fs.readFileSync(p, "utf8"));
}

function writeJson(p, obj) {
    fs.writeFileSync(p, `${JSON.stringify(obj, null, 2)}\n`);
}

function copyTree(src, dest, filter) {
    fs.mkdirSync(dest, { recursive: true });
    for (const name of fs.readdirSync(src)) {
        if (name === "node_modules" || name === ".git") continue;
        const from = path.join(src, name);
        const to = path.join(dest, name);
        const st = fs.statSync(from);
        if (st.isDirectory()) {
            if (filter && !filter(from, true)) continue;
            copyTree(from, to, filter);
        } else {
            if (filter && !filter(from, false)) continue;
            fs.mkdirSync(path.dirname(to), { recursive: true });
            fs.copyFileSync(from, to);
        }
    }
}

function rewriteWorkspaceDeps(deps, version) {
    if (!deps) return deps;
    /** @type {Record<string, string>} */
    const out = {};
    for (const [k, v] of Object.entries(deps)) {
        if (typeof v === "string" && (v.startsWith("workspace:") || v.startsWith("file:") || v === "*")) {
            out[k] = version;
        } else {
            out[k] = v;
        }
    }
    return out;
}

function rewriteDepsField(pkg, version) {
    for (const field of ["dependencies", "optionalDependencies", "peerDependencies"]) {
        if (pkg[field]) pkg[field] = rewriteWorkspaceDeps(pkg[field], version);
    }
    return pkg;
}

function isAlreadyPublished(blob) {
    return /cannot publish over existing|EPUBLISHCONFLICT|previously published versions|version already exists|cannot publish.*same version|you cannot publish over/i.test(
        blob,
    );
}

function isAuthFailure(blob) {
    return /ENEEDAUTH|Unable to authenticate|not authorized|OIDC|trusted publisher|two-factor|need to be logged|login|identity token|do not have permission to access it|Access token expired or revoked/i.test(
        blob,
    );
}

function isMissingPackage(blob) {
    if (isAuthFailure(blob)) return false;
    return /Package not found|does not exist on the registry|cannot publish.*before creating|This package has not been created|is not in this registry/i.test(
        blob,
    );
}

function versionExists(name, version) {
    const r = run("npm", ["view", `${name}@${version}`, "version"]);
    return r.status === 0 && r.stdout === version;
}

function isLowerThanLatestTagError(blob) {
    return /Cannot implicitly apply the "latest" tag because previously published version/i.test(blob);
}

/**
 * @param {string} stagingDir
 * @param {string} name
 * @param {string} version
 * @param {{ npmTag?: string }} [opts]
 * @returns {"published"|"exists"|"auth"|"missing"|"other"}
 */
function npmPublish(stagingDir, name, version, opts = {}) {
    const tags = opts.npmTag ? [opts.npmTag] : [undefined, `v${version}`];
    for (const npmTag of tags) {
        const args = ["publish", "--access", "public"];
        if (npmTag) args.push("--tag", npmTag);
        console.log(`\n=== ${name}@${version} npm ${args.join(" ")} ===`);
        const r = run("npm", args, { cwd: stagingDir });
        if (r.stdout) process.stdout.write(`${r.stdout}\n`);
        if (r.stderr) process.stderr.write(`${r.stderr}\n`);
        const blob = `${r.stdout}\n${r.stderr}`;
        if (r.status === 0) return "published";
        if (isAlreadyPublished(blob) || versionExists(name, version)) return "exists";
        if (isAuthFailure(blob)) return "auth";
        if (isMissingPackage(blob)) return "missing";
        if (versionExists(name, version)) return "exists";
        if (!npmTag && isLowerThanLatestTagError(blob)) {
            console.log(` retrying ${name}@${version} with --tag v${version}`);
            continue;
        }
        console.error(blob.slice(0, 1200));
        return "other";
    }
    return "other";
}

/** Find a .node under artifact dir (flat or nested lib/). */
function findNodeInArtifact(artDir, wantFile) {
    if (!fs.existsSync(artDir)) return undefined;
    const direct = path.join(artDir, wantFile);
    if (fs.existsSync(direct)) return direct;
    const lib = path.join(artDir, "lib", wantFile);
    if (fs.existsSync(lib)) return lib;
    /** @type {string | undefined} */
    let fallback;
    const walk = (dir) => {
        for (const name of fs.readdirSync(dir)) {
            const full = path.join(dir, name);
            if (fs.statSync(full).isDirectory()) {
                walk(full);
            } else if (name.endsWith(".node")) {
                if (name === wantFile) fallback = full;
                else if (!fallback) fallback = full;
            }
        }
    };
    walk(artDir);
    return fallback;
}

/**
 * @param {string} version
 * @param {string} artifactsRoot
 */
function publishNative(version, artifactsRoot) {
    let published = 0;
    let skipped = 0;
    for (const plat of NATIVE_PLATFORMS) {
        const name = `@doki-land/nifty-${plat.short}`;
        const artDir = path.join(artifactsRoot, plat.short);
        if (versionExists(name, version)) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }
        const nodeSrc = findNodeInArtifact(artDir, plat.nodeFile);
        if (!nodeSrc) {
            console.log(` · ${name} no artifact (${plat.short}) — skip`);
            skipped += 1;
            continue;
        }

        const stage = path.join(os.tmpdir(), `nifty-pub-native-${plat.short}-${version}`);
        fs.rmSync(stage, { recursive: true, force: true });
        fs.mkdirSync(stage, { recursive: true });

        const sourceAbs = path.join(ROOT, plat.sourceDir);
        fs.copyFileSync(path.join(sourceAbs, "index.js"), path.join(stage, "index.js"));
        const libDir = path.join(stage, "lib");
        fs.mkdirSync(libDir, { recursive: true });
        fs.copyFileSync(nodeSrc, path.join(libDir, plat.nodeFile));

        const raw = readJson(path.join(sourceAbs, "package.json"));
        writeJson(path.join(stage, "package.json"), {
            name,
            version,
            description: raw.description ?? `Nifty Node-API binary (${plat.short})`,
            type: "module",
            license: raw.license ?? "MPL-2.0",
            main: "./index.js",
            files: ["index.js", "lib"],
            os: plat.os,
            cpu: plat.cpu,
            publishConfig: { access: "public" },
            repository: { type: "git", url: REPO_URL },
        });

        const outcome = npmPublish(stage, name, version, { npmTag: `v${version}` });
        if (outcome === "published") published += 1;
        else if (outcome === "exists") {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
        } else if (outcome === "auth") {
            fail(`OIDC/auth failed for ${name}. Trusted Publisher: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/npm-tools`);
        } else fail(`publish failed for ${name}`);
    }
    return { published, skipped };
}

/**
 * @param {string} version
 * @param {string} artifactsRoot
 */
function publishJs(version, artifactsRoot) {
    let published = 0;
    let skipped = 0;
    /** @type {Record<string, string>} */
    const optionalNatives = {};
    for (const plat of NATIVE_PLATFORMS) {
        const n = `@doki-land/nifty-${plat.short}`;
        const artDir = path.join(artifactsRoot, plat.short);
        if (findNodeInArtifact(artDir, plat.nodeFile) || versionExists(n, version)) {
            optionalNatives[n] = version;
        } else {
            console.log(` · ${n}@${version} not built this release — omit from optionalDependencies`);
        }
    }

    for (const spec of JS_PACKAGES) {
        const abs = path.join(ROOT, spec.dir);
        if (!fs.existsSync(abs)) fail(`missing package dir ${spec.dir}`);
        const raw = readJson(path.join(abs, "package.json"));
        const name = spec.publishName ?? raw.name;
        if (!name) fail(`no name for ${spec.dir}`);

        if (versionExists(name, version)) {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
            continue;
        }

        const stage = path.join(os.tmpdir(), `nifty-pub-js-${name.replace(/[/@]/g, "-")}-${version}`);
        fs.rmSync(stage, { recursive: true, force: true });
        fs.mkdirSync(stage, { recursive: true });

        const files = Array.isArray(raw.files) && raw.files.length ? raw.files : null;
        if (files) {
            for (const f of files) {
                const from = path.join(abs, f);
                if (!fs.existsSync(from)) continue;
                const to = path.join(stage, f);
                if (fs.statSync(from).isDirectory()) copyTree(from, to);
                else {
                    fs.mkdirSync(path.dirname(to), { recursive: true });
                    fs.copyFileSync(from, to);
                }
            }
            for (const extra of ["package.json", "README.md", "LICENSE", "bin"]) {
                const from = path.join(abs, extra);
                if (!fs.existsSync(from)) continue;
                const to = path.join(stage, extra);
                if (fs.statSync(from).isDirectory()) copyTree(from, to);
                else fs.copyFileSync(from, to);
            }
        } else {
            copyTree(abs, stage, (p) => {
                const rel = path.relative(abs, p);
                if (rel.includes("node_modules") || rel.endsWith(".node")) return false;
                return true;
            });
        }

        const pkg = rewriteDepsField({ ...raw }, version);
        pkg.name = name;
        pkg.version = version;
        delete pkg.private;
        pkg.publishConfig = { ...(pkg.publishConfig ?? {}), access: "public" };
        if (pkg.scripts) {
            delete pkg.scripts.prepack;
            delete pkg.scripts.prepare;
            if (Object.keys(pkg.scripts).length === 0) delete pkg.scripts;
        }
        if (!pkg.repository) {
            pkg.repository = { type: "git", url: REPO_URL };
        }
        pkg.optionalDependencies = { ...optionalNatives };
        delete pkg.devDependencies;
        writeJson(path.join(stage, "package.json"), pkg);

        const outcome = npmPublish(stage, name, version);
        if (outcome === "published") published += 1;
        else if (outcome === "exists") {
            console.log(` ✓ ${name}@${version} already on registry — skip`);
            skipped += 1;
        } else if (outcome === "auth") {
            fail(`OIDC/auth failed for ${name}. Trusted Publisher: file=publish-npm.yml env=NPM_PUBLISH repo=oovm/npm-tools`);
        } else fail(`publish failed for ${name}`);
    }
    return { published, skipped };
}

const version = resolveVersion();
console.log(`ci-publish-npm: version=${version}`);
console.log(` GITHUB_REF=${process.env.GITHUB_REF ?? "(none)"}`);
console.log(" Trusted Publisher contract: publish-npm.yml + env NPM_PUBLISH\n");

delete process.env.NODE_AUTH_TOKEN;
delete process.env.NPM_TOKEN;

const artifactsRoot = process.env.NIFTY_NATIVE_ARTIFACTS || path.join(ROOT, "dist", "native-flat");

const native = publishNative(version, artifactsRoot);
const js = publishJs(version, artifactsRoot);

console.log(
    `\nci-publish-npm: done (native published=${native.published} skipped=${native.skipped}; js published=${js.published} skipped=${js.skipped})`,
);
