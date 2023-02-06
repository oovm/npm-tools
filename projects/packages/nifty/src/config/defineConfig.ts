import type { NiftyConfig, NiftyConfigEnv } from "./types.js";

/** Type-safe helper for `nifty.config.ts` / `nifty.config.js`. */
export function defineConfig(config: NiftyConfig): NiftyConfig;
export function defineConfig(
    config: (env: NiftyConfigEnv) => NiftyConfig | Promise<NiftyConfig>,
): (env: NiftyConfigEnv) => NiftyConfig | Promise<NiftyConfig>;
export function defineConfig(
    config: NiftyConfig | ((env: NiftyConfigEnv) => NiftyConfig | Promise<NiftyConfig>),
): NiftyConfig | ((env: NiftyConfigEnv) => NiftyConfig | Promise<NiftyConfig>) {
    return config;
}
