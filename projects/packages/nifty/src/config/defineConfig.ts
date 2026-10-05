import type { NiftyConfigExport } from "./types.js";

/** Type-safe helper for `nifty.config.ts` / `nifty.config.js`. */
export function defineConfig(config: NiftyConfigExport): NiftyConfigExport {
    return config;
}
