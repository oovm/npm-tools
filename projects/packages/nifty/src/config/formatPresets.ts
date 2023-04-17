import type { NiftyFormatConfig, NiftyFormatPreset } from "./types.js";

export type ResolvedFormatOptions = {
    preset: NiftyFormatPreset;
    includes?: string[];
    excludes?: string[];
    rust?: boolean;
    javascript?: boolean;
    style?: string;
};

const NPM_TOOLS_INCLUDES = [
    "scripts/**",
    "projects/packages/**",
    "projects/conformance/**",
    "projects/dashboard/**",
    "package.json",
    "biome.json",
    "nifty.config.ts",
] as const;

const PRESET_INCLUDES: Record<NiftyFormatPreset, readonly string[] | undefined> = {
    default: undefined,
    "npm-tools": NPM_TOOLS_INCLUDES,
};

/** Merge `nifty.config` `format` with built-in preset defaults. */
export function resolveFormatConfig(format: NiftyFormatConfig | undefined): ResolvedFormatOptions {
    const preset = format?.preset ?? "default";
    const presetIncludes = PRESET_INCLUDES[preset];
    return {
        preset,
        includes: format?.includes ?? (presetIncludes ? [...presetIncludes] : undefined),
        excludes: format?.excludes,
        rust: format?.rust,
        javascript: format?.javascript,
        style: format?.style,
    };
}
