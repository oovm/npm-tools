import { spawn } from "node:child_process";
import { createHmac } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join, resolve } from "node:path";

import { detectProjectLayout } from "../config/detectProject.js";

const DEFAULT_REGISTRY = "https://registry.npmjs.org";
const SECRETS_FILE = ".secrets/nifty.json";
const LOGIN_TIMEOUT_MS = 5 * 60 * 1000;
const POLL_FALLBACK_MS = 2000;

/** One npm web-login handshake: open `loginUrl`, poll `doneUrl` for the token. */
export type LoginSession = {
    loginUrl: string;
    doneUrl: string;
};

export type LoginTokenOptions = {
    timeoutMs?: number;
};

/** Login writes the same workspace root the publisher reads secrets from. */
export function findLoginRoot(cwd: string): string {
    const layout = detectProjectLayout(cwd);
    for (const candidate of [layout.root, resolve(cwd)]) {
        const hybrid =
            existsSync(join(candidate, "projects", "crates")) && existsSync(join(candidate, "projects", "packages"));
        if (hybrid || existsSync(join(candidate, "pnpm-workspace.yaml"))) {
            return candidate;
        }
    }
    throw new Error("could not find workspace root (expected a Nifty hybrid workspace or pnpm-workspace.yaml)");
}

/** Registry used for login (CLI flag, then npm env config, then npmjs). */
export function resolveRegistry(explicit?: string): string {
    const raw = explicit ?? process.env.npm_config_registry ?? DEFAULT_REGISTRY;
    return raw.trim().replace(/\/+$/, "");
}

/** `POST /-/v1/login` — the endpoint behind `npm login --auth-type=web`. */
export async function startWebLogin(registry: string): Promise<LoginSession> {
    const response = await fetch(`${registry}/-/v1/login`, {
        method: "POST",
        headers: { "content-type": "application/json", "npm-auth-type": "web" },
        body: "{}",
    });
    const text = await response.text();
    if (response.status !== 200) {
        throw new Error(`web login is not available on ${registry} (HTTP ${response.status}): ${text.trim()}`);
    }
    const body = JSON.parse(text) as { loginUrl?: unknown; doneUrl?: unknown };
    if (typeof body.loginUrl !== "string" || typeof body.doneUrl !== "string") {
        throw new Error(`unexpected login response from ${registry}: ${text.trim()}`);
    }
    return { loginUrl: body.loginUrl, doneUrl: body.doneUrl };
}

/** Poll `doneUrl` (202 pending, 200 token) until the browser approves or timeout. */
export async function waitForLoginToken(doneUrl: string, options: LoginTokenOptions = {}): Promise<string> {
    const deadline = Date.now() + (options.timeoutMs ?? LOGIN_TIMEOUT_MS);
    while (true) {
        const response = await fetch(doneUrl, { headers: { "cache-control": "no-cache" } });
        const text = await response.text();
        if (response.status === 200) {
            const body = JSON.parse(text) as { token?: unknown };
            if (typeof body.token === "string" && body.token.length > 0) {
                return body.token;
            }
            throw new Error(`login response is missing a token: ${text.trim()}`);
        }
        if (response.status !== 202) {
            throw new Error(`login failed (HTTP ${response.status}): ${text.trim()}`);
        }
        const retryAfter = Number(response.headers.get("retry-after"));
        const waitMs = Number.isFinite(retryAfter) && retryAfter > 0 ? retryAfter * 1000 : POLL_FALLBACK_MS;
        const remaining = deadline - Date.now();
        if (remaining <= 0) {
            throw new Error("login timed out waiting for browser approval; run `nifty login` again");
        }
        await sleep(Math.min(waitMs, remaining));
    }
}

/** Verify a token the same way npm does and return the account name. */
export async function whoami(registry: string, token: string): Promise<string> {
    const response = await fetch(`${registry}/-/whoami`, {
        headers: { authorization: `Bearer ${token}` },
    });
    const text = await response.text();
    if (response.status !== 200) {
        throw new Error(`token rejected by ${registry} (HTTP ${response.status}): ${text.trim()}`);
    }
    const body = JSON.parse(text) as { username?: unknown };
    if (typeof body.username !== "string" || body.username.length === 0) {
        throw new Error(`whoami response is missing a username: ${text.trim()}`);
    }
    return body.username;
}

/** Open the npm web-login page in the default browser (best effort). */
export function openInBrowser(url: string): void {
    const [command, args] =
        process.platform === "win32"
            ? ["cmd", ["/c", "start", "", url]]
            : process.platform === "darwin"
              ? ["open", [url]]
              : ["xdg-open", [url]];
    const child = spawn(command, args, { stdio: "ignore", detached: true, windowsHide: true });
    child.on("error", () => {
        // Opening is a convenience; the URL is always printed.
    });
    child.unref();
}

/** RFC 6238 TOTP (SHA-1, 6 digits, 30s) matching npm 2FA codes. */
export function totpCode(secret: string, atMs = Date.now()): string {
    const key = decodeBase32(secret);
    const counter = Math.floor(atMs / 1000 / 30);
    const buffer = Buffer.alloc(8);
    buffer.writeBigUInt64BE(BigInt(counter));
    const digest = createHmac("sha1", key).update(buffer).digest();
    const offset = digest[digest.length - 1]! & 0x0f;
    const value =
        ((digest[offset]! & 0x7f) << 24) |
        (digest[offset + 1]! << 16) |
        (digest[offset + 2]! << 8) |
        digest[offset + 3]!;
    return String(value % 1_000_000).padStart(6, "0");
}

/** Seconds the current TOTP code still stays valid. */
export function totpSecondsLeft(atMs = Date.now()): number {
    return 30 - (Math.floor(atMs / 1000) % 30);
}

export function secretsPath(root: string): string {
    return join(root, SECRETS_FILE);
}

/** Read `.secrets/nifty.json` as-is; missing file reads as `{}`. */
export function readSecrets(root: string): Record<string, unknown> {
    const path = secretsPath(root);
    if (!existsSync(path)) {
        return {};
    }
    const text = readFileSync(path, "utf8");
    let parsed: unknown;
    try {
        parsed = JSON.parse(text);
    } catch (error) {
        throw new Error(`invalid ${path}: ${error instanceof Error ? error.message : String(error)}`);
    }
    if (parsed === null || typeof parsed !== "object" || Array.isArray(parsed)) {
        throw new Error(`invalid ${path}: expected a JSON object`);
    }
    return parsed as Record<string, unknown>;
}

/** npm's user config file: `npm login` persists the token here too. */
export function npmRcPath(): string {
    return process.env.npm_config_userconfig ?? join(homedir(), ".npmrc");
}

/** `_authToken` key npm uses for a registry (`//host/path/:_authToken`). */
export function authTokenKey(registry: string): string {
    const url = new URL(registry);
    const path = url.pathname.endsWith("/") ? url.pathname : `${url.pathname}/`;
    return `//${url.host}${path}:_authToken`;
}

/** Store the token in the npm user config, replacing a stale one in place. */
export function writeNpmToken(registry: string, token: string): string {
    const path = npmRcPath();
    const key = authTokenKey(registry);
    const existing = existsSync(path) ? readFileSync(path, "utf8") : "";
    const lines: string[] = [];
    for (const line of existing.split(/\r?\n/)) {
        const separator = line.indexOf("=");
        const lineKey = separator === -1 ? "" : line.slice(0, separator).trim();
        if (lineKey === key) {
            continue;
        }
        lines.push(line);
    }
    while (lines.length > 0 && lines[lines.length - 1]!.trim() === "") {
        lines.pop();
    }
    lines.push(`${key}=${token}`);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, `${lines.join("\n")}\n`, "utf8");
    return path;
}

export function totpSecretFrom(secrets: Record<string, unknown>): string | undefined {
    const value = secrets.npmTotpSecret;
    return typeof value === "string" && value.length > 0 ? value : undefined;
}

function decodeBase32(secret: string): Buffer {
    const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    const cleaned = secret.replace(/[\s=-]/g, "").toUpperCase();
    let bits = "";
    for (const char of cleaned) {
        const index = alphabet.indexOf(char);
        if (index === -1) {
            throw new Error("invalid base32 in TOTP secret");
        }
        bits += index.toString(2).padStart(5, "0");
    }
    const bytes: number[] = [];
    for (let offset = 0; offset + 8 <= bits.length; offset += 8) {
        bytes.push(Number.parseInt(bits.slice(offset, offset + 8), 2));
    }
    if (bytes.length === 0) {
        throw new Error("TOTP secret decoded empty");
    }
    return Buffer.from(bytes);
}

function sleep(ms: number): Promise<void> {
    return new Promise((resolveSleep) => {
        setTimeout(resolveSleep, ms);
    });
}
