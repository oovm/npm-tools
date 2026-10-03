import type { NiftyConfig } from "./types.js";

/** Map `nifty.config` `trust` into native publisher options. */
export function trustPayloadFromConfig(config: NiftyConfig) {
    const trust = config.trust;
    if (!trust) {
        return {};
    }
    return {
        ...(trust.repo ? { trustRepo: trust.repo } : {}),
        ...(trust.file ? { trustFile: trust.file } : {}),
        ...(trust.environment ? { trustEnvironment: trust.environment } : {}),
    };
}
