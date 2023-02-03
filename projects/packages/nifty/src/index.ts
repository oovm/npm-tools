/**
 * @doki-land/nifty — gitmoji-first commit conventions and gix-backed git helpers.
 *
 * Build the WASI component first: `npm run build` in this package.
 */

export type {
    CommitRecord,
    GithubAuthor,
    ParsedSubject,
    RangeInfo,
    ReleaseSection,
    TagInfo,
} from "./types.js";
export { Git, createGit, type ResolveRangeOptions } from "./git.js";
export { Github, createGithub, type LookupUserOptions } from "./github.js";
export { Gitmoji, createGitmoji } from "./gitmoji.js";
export { Nifty, createNifty, type NiftyOpenOptions } from "./nifty.js";
export {
    defineConfig,
    loadConfig,
    findConfigFile,
    authorMapToJson,
    detectProjectLayout,
    findCargoManifest,
    findPackageManifest,
    type LoadConfigOptions,
    type LoadedNiftyConfig,
    type NiftyAuthorEntry,
    type NiftyConfig,
    type NiftyConfigEnv,
    type ProjectKind,
    type ProjectLayout,
} from "@doki-land/nifty-config";
export { update, type UpdateOptions } from "@doki-land/nifty-updater";
export {
    lint,
    check,
    lintSubjects,
    DEFAULT_RULES,
    type LintDiagnostic,
    type LintOptions,
    type LintReport,
    type LintRuleConfig,
    type LintSeverity,
} from "@doki-land/nifty-linter";
