export { defineConfig } from "./defineConfig.js";
export {
    detectProjectLayout,
    discoverPackageDirs,
    findCargoManifest,
    findPackageManifest,
    type ProjectKind,
    type ProjectLayout,
} from "./detectProject.js";
export {
    authorMapToJson,
    defaultConfigPath,
    ensureConfigFile,
    findConfigFile,
    loadConfig,
    type LoadConfigOptions,
    type LoadedNiftyConfig,
} from "./loadConfig.js";
export { resolveFormatConfig, type ResolvedFormatOptions } from "./formatPresets.js";
export {
    CONFIG_FILE_NAMES,
    type NiftyAuthorEntry,
    type NiftyConfig,
    type NiftyConfigEnv,
    type NiftyConfigExport,
    type NiftyFormatConfig,
    type NiftyFormatPreset,
} from "./types.js";
