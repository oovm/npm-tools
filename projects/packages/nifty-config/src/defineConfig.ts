import type { NiftyConfig } from "./types.js";

/** Type-safe helper for `nifty.config.ts` / `nifty.config.js`. */
export function defineConfig(config: NiftyConfig): NiftyConfig;
export function defineConfig(config: Promise<NiftyConfig>): Promise<NiftyConfig>;
export function defineConfig(config: NiftyConfig | Promise<NiftyConfig>): NiftyConfig | Promise<NiftyConfig> {
    return config;
}
