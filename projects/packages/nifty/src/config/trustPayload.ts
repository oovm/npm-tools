import type { NiftyConfig } from "./types.js";

/** Map `nifty.config` `trust.npm` into native publisher options. */
export function trustPayloadFromConfig(config: NiftyConfig) {
    const npm = config.trust?.npm;
    if (!npm) {
        return {};
    }
    return {
        ...(npm.repo !== undefined ? { trustRepo: npm.repo } : {}),
        ...(npm.file !== undefined ? { trustFile: npm.file } : {}),
        ...(npm.environment !== undefined ? { trustEnvironment: npm.environment } : {}),
    };
}
