import type { NiftyFormatConfig, NiftyFormatPreset, NiftyFormatStyleConfig } from "./types.js";

export type ResolvedFormatOptions = {
    preset: NiftyFormatPreset;
    includes?: string[];
    excludes?: string[];
    rust?: boolean;
    javascript?: boolean;
    style: NiftyFormatStyleConfig;
};

const DEFAULT_FORMAT_STYLE: NiftyFormatStyleConfig = {
    indentStyle: "space",
    indentWidth: 4,
    lineWidth: 144,
    quoteStyle: "single",
};

const NPM_TOOLS_INCLUDES = [
    "scripts/**",
    "projects/packages/**",
    "projects/conformance/**",
    "projects/dashboard/**",
    "package.json",
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
        style: {
            ...DEFAULT_FORMAT_STYLE,
            ...format?.style,
        },
    };
}
