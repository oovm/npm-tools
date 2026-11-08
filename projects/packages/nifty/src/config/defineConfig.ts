import type { NiftyConfigExport } from "./types.js";

/**
 * Type-safe helper for `nifty.config.ts` / `nifty.config.js`.
 *
 * A single union parameter keeps the diagnostics on the offending argument. A conditional
 * input type instead collapsed mismatches into `never`, so an unknown key only reported
 * `Type '{}' is not assignable to type 'never'` and never named the key or the call form.
 * With `NiftyConfigExport` an unknown key reports
 * `Object literal may only specify known properties, and '<key>' does not exist in type 'NiftyConfigExport'`,
 * and a value that is neither an object nor a factory reports
 * `Argument of type '<type>' is not assignable to parameter of type 'NiftyConfigExport'`.
 */
export function defineConfig(config: NiftyConfigExport): NiftyConfigExport {
    return config;
}
