import type { CommitRecord, RangeInfo, TagInfo } from "./types.js";
import { loadNiftyWasm, mapCommitRecord, mapRangeInfo, mapTagInfo } from "./wasm.js";

export type ResolveRangeOptions = {
    version?: string;
    fromRef?: string;
    toRef?: string;
};

/**
 * Read-only git repository helpers (gix inside WASI, no `git` subprocess).
 */
export class Git {
    constructor(private readonly wasm: Awaited<ReturnType<typeof loadNiftyWasm>>["git"]) {}

    static async open(): Promise<Git> {
        const wasm = await loadNiftyWasm();
        return new Git(wasm.git);
    }

    /** Walk upward from `startPath` to find the working tree root containing `.git`. */
    discoverRoot(startPath: string): string {
        return this.wasm["discover-root"](startPath);
    }

    /** List `v*` tags sorted by semver. */
    listVersionTags(repoRoot: string): string[] {
        return this.wasm["list-version-tags"](repoRoot);
    }

    listTagInfos(repoRoot: string): TagInfo[] {
        return this.wasm["list-tag-infos"](repoRoot).map(mapTagInfo);
    }

    formatTagList(repoRoot: string): string {
        return this.wasm["format-tag-list"](repoRoot);
    }

    resolveRange(repoRoot: string, options: ResolveRangeOptions = {}): RangeInfo {
        return mapRangeInfo(
            this.wasm["resolve-range"](repoRoot, options.version, options.fromRef, options.toRef),
        );
    }

    /** Collect non-merge commits in `fromRef..toRef` (exclusive..inclusive). */
    collectCommits(repoRoot: string, fromRef: string | undefined, toRef: string): CommitRecord[] {
        return this.wasm["collect-commits"](repoRoot, fromRef, toRef).map(mapCommitRecord);
    }

    detectGithubRepo(repoRoot: string): string | undefined {
        return this.wasm["detect-github-repo"](repoRoot);
    }

    parseGithubRemote(url: string): string | undefined {
        return this.wasm["parse-github-remote"](url);
    }
}

export async function createGit(): Promise<Git> {
    return Git.open();
}
