import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { createJiti } from "jiti";

import type { NiftyConfig, NiftyConfigEnv, NiftyConfigExport } from "./types.js";
import { CONFIG_FILE_NAMES } from "./types.js";
import { detectProjectLayout } from "./detectProject.js";

export type LoadConfigOptions = {
    /** Directory to start searching from (defaults to `process.cwd()`). */
    cwd?: string;
    /** When set, skip searching and load this file directly. */
    configFile?: string;
    /** Passed to functional configs: `export default defineConfig(({ mode }) => ...)`. */
    env?: Partial<NiftyConfigEnv>;
};

export type LoadedNiftyConfig = {
    config: NiftyConfig;
    configFile?: string;
};

const DEFAULT_ENV = (cwd: string): NiftyConfigEnv => ({
    mode: process.env.NODE_ENV ?? "development",
    command: "serve",
    layout: detectProjectLayout(cwd),
});

/** Walk upward and return the nearest existing Nifty config file. */
export function findConfigFile(cwd = process.cwd()): string | undefined {
    let here = resolve(cwd);
    while (true) {
        for (const name of CONFIG_FILE_NAMES) {
            const candidate = join(here, name);
            if (existsSync(candidate)) {
                return candidate;
            }
        }
        const parent = dirname(here);
        if (parent === here) {
            return undefined;
        }
        here = parent;
    }
}

function normalizeExport(raw: unknown): NiftyConfigExport {
    if (raw && typeof raw === "object" && "default" in raw) {
        return (raw as { default: NiftyConfigExport }).default;
    }
    return raw as NiftyConfigExport;
}

async function resolveExport(exported: NiftyConfigExport, env: NiftyConfigEnv): Promise<NiftyConfig> {
    if (typeof exported === "function") {
        return await exported(env);
    }
    return exported;
}

/** Load `nifty.config.ts/js` from `cwd` or the given `configFile`. */
export async function loadConfig(options: LoadConfigOptions = {}): Promise<LoadedNiftyConfig> {
    const cwd = options.cwd ?? process.cwd();
    const configFile = options.configFile ?? findConfigFile(cwd);
    if (!configFile) {
        return { config: {} };
    }

    const env: NiftyConfigEnv = { ...DEFAULT_ENV(cwd), ...options.env };
    const jiti = createJiti(import.meta.url, { interopDefault: true });
    const mod = await jiti.import(configFile);
    const config = await resolveExport(normalizeExport(mod), env);
    return { config, configFile };
}

/** Serialize `authorMap` for Rust/WASI helpers expecting JSON text. */
export function authorMapToJson(authorMap: NiftyConfig["authorMap"]): string {
    if (!authorMap || Object.keys(authorMap).length === 0) {
        return "{}";
    }
    return JSON.stringify(authorMap);
}
