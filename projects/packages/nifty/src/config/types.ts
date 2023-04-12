/** GitHub author entry in `authorMap` (same shape as `author-github.json`). */
export type NiftyAuthorEntry = {
    id?: number;
    login?: string;
};

/** Built-in workspace format presets (`nifty.config` `format.preset`). */
export type NiftyFormatPreset = "default" | "npm-tools" | "vmz";

/** Workspace format options (`nifty format` reads `nifty.config` `format`). */
export type NiftyFormatConfig = {
    /** Select built-in include roots. Use `vmz` for the VMZ hybrid monorepo. */
    preset?: NiftyFormatPreset;
    /** Extra include globs (override preset list when set alone). */
    includes?: string[];
    /** Skip paths under these globs after includes expand. */
    excludes?: string[];
    /** Run `cargo fmt` (default: true when a Cargo workspace is detected). */
    rust?: boolean;
    /** Run `oxc_formatter` on JS/TS targets (default: true). */
    javascript?: boolean;
    /** Biome-style formatter config for oxc (default: `biome.json` at project root). */
    style?: string;
};

/** Nifty project configuration (`nifty.config.ts` / `nifty.config.js`). */
export type NiftyConfig = {
    /** GitHub personal access token for email search / profile fetch. */
    githubToken?: string;
    /** Email → GitHub author mapping. */
    authorMap?: Record<string, NiftyAuthorEntry>;
    /** Repository root override when discovering git metadata. */
    repoRoot?: string;
    /** Cargo workspace / crate root override. */
    cargoRoot?: string;
    /** npm package root override. */
    npmRoot?: string;
    /** npm publish / Trusted Publisher (CI uses publish-npm.yml). */
    publish?: {
        /** Package names to trust (includes registry-only native sidecars). */
        packages?: string[];
    };
    /** Release reference changelog defaults (`nifty change-logs`). */
    changelog?: {
        /** GitHub `owner/repo` for contrib.rocks (default: parse `origin`). */
        repo?: string;
        /** Email → GitHub map path (default: `documentation/maintenance/author-github.json`). */
        authorMap?: string;
        /** `--write` output directory (default: `documentation/maintenance/releases`). */
        releasesDir?: string;
    };
    /** Rule-based lint configuration. */
    lint?: {
        rules?: Array<{
            id: string;
            enabled?: boolean;
            severity?: "error" | "warning" | "info";
        }>;
    };
    /** Workspace format targets and engines (`nifty format`). */
    format?: NiftyFormatConfig;
};

export type NiftyConfigExport = NiftyConfig | ((env: NiftyConfigEnv) => NiftyConfig | Promise<NiftyConfig>);

export type NiftyConfigEnv = {
    mode: string;
    command: string;
    /** Detected cargo + npm project layout for hybrid monorepos. */
    layout: import("./detectProject.js").ProjectLayout;
};

export const CONFIG_FILE_NAMES = [
    "nifty.config.ts",
    "nifty.config.js",
    "nifty.config.mjs",
    "nifty.config.cjs",
] as const;
