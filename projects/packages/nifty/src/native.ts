import { createRequire } from "node:module";
import type {
    CommitRecord,
    GithubAuthor,
    ParsedSubject,
    RangeInfo,
    ReleaseSection,
    TagInfo,
} from "./types.js";

export type GitmojiExports = {
    "known-gitmojis": () => string[];
    "validate-subject": (subject: string) => boolean;
    "parse-subject": (subject: string) => ParsedSubject;
    "format-subject": (gitmoji: string, body: string) => string;
    "strip-gitmoji": (subject: string) => string;
    "leading-gitmoji": (subject: string) => string | undefined;
    "section-for-gitmoji": (gitmoji: string | undefined) => ReleaseSection;
    "parse-noreply-email": (email: string) => GithubAuthor | undefined;
    "resolve-github-author": (email: string, authorMapJson: string) => GithubAuthor | undefined;
    "display-login": (author: GithubAuthor) => string;
    "profile-url": (author: GithubAuthor) => string;
    "avatar-url": (author: GithubAuthor) => string;
    "author-mention": (email: string, authorName: string, authorMapJson: string) => string;
    "commit-bullet": (body: string, email: string, authorName: string, authorMapJson: string) => string;
};

export type GitExports = {
    "discover-root": (startPath: string) => string;
    "list-version-tags": (repoRoot: string) => string[];
    "list-tag-infos": (repoRoot: string) => Array<{ name: string; "short-hash": string }>;
    "format-tag-list": (repoRoot: string) => string;
    "resolve-range": (
        repoRoot: string,
        version: string | undefined,
        fromRef: string | undefined,
        toRef: string | undefined,
    ) => { version: string; "from-ref"?: string; "to-ref": string };
    "collect-commits": (
        repoRoot: string,
        fromRef: string | undefined,
        toRef: string,
    ) => Array<{
        hash: string;
        email: string;
        author: string;
        subject: string;
        body: string;
        gitmoji?: string;
        section: string;
    }>;
    "detect-github-repo": (repoRoot: string) => string | undefined;
    "parse-github-remote": (url: string) => string | undefined;
};

export type NiftyNative = {
    gitmoji: GitmojiExports;
    git: GitExports;
};

type NativeBinding = {
    gitmojiKnownGitmojis: () => string[];
    gitmojiValidateSubject: (subject: string) => boolean;
    gitmojiParseSubject: (subject: string) => ParsedSubject;
    gitmojiFormatSubject: (gitmoji: string, body: string) => string;
    gitmojiStripGitmoji: (subject: string) => string;
    gitmojiLeadingGitmoji: (subject: string) => string | null;
    gitmojiSectionForGitmoji: (gitmoji?: string | null) => string;
    gitmojiParseNoreplyEmail: (email: string) => GithubAuthor | null;
    gitmojiResolveGithubAuthor: (email: string, authorMapJson: string) => GithubAuthor | null;
    gitmojiDisplayLogin: (author: GithubAuthor) => string;
    gitmojiProfileUrl: (author: GithubAuthor) => string;
    gitmojiAvatarUrl: (author: GithubAuthor) => string;
    gitmojiAuthorMention: (email: string, authorName: string, authorMapJson: string) => string;
    gitmojiCommitBullet: (body: string, email: string, authorName: string, authorMapJson: string) => string;
    gitDiscoverRoot: (startPath: string) => string;
    gitListVersionTags: (repoRoot: string) => string[];
    gitListTagInfos: (repoRoot: string) => Array<{ name: string; shortHash: string }>;
    gitFormatTagList: (repoRoot: string) => string;
    gitResolveRange: (
        repoRoot: string,
        version?: string | null,
        fromRef?: string | null,
        toRef?: string | null,
    ) => { version: string; fromRef?: string; toRef: string };
    gitCollectCommits: (
        repoRoot: string,
        fromRef: string | null | undefined,
        toRef: string,
    ) => CommitRecord[];
    gitDetectGithubRepo: (repoRoot: string) => string | null;
    gitParseGithubRemote: (url: string) => string | null;
};

const PLATFORM_PACKAGES: Record<string, string> = {
    "win32-x64": "@doki-land/nifty-win32-x64",
    "linux-x64": "@doki-land/nifty-linux-x64",
    "linux-arm64": "@doki-land/nifty-linux-arm64",
    "darwin-x64": "@doki-land/nifty-darwin-x64",
    "darwin-arm64": "@doki-land/nifty-darwin-arm64",
};

function mapAuthor(raw: GithubAuthor | null | undefined): GithubAuthor | undefined {
    if (!raw) {
        return undefined;
    }
    return {
        id: raw.id !== undefined && raw.id !== null ? BigInt(raw.id) : undefined,
        login: raw.login ?? undefined,
    };
}

function wrapBinding(binding: NativeBinding): NiftyNative {
    return {
        gitmoji: {
            "known-gitmojis": () => binding.gitmojiKnownGitmojis(),
            "validate-subject": (subject) => binding.gitmojiValidateSubject(subject),
            "parse-subject": (subject) => binding.gitmojiParseSubject(subject),
            "format-subject": (gitmoji, body) => binding.gitmojiFormatSubject(gitmoji, body),
            "strip-gitmoji": (subject) => binding.gitmojiStripGitmoji(subject),
            "leading-gitmoji": (subject) => binding.gitmojiLeadingGitmoji(subject) ?? undefined,
            "section-for-gitmoji": (gitmoji) => binding.gitmojiSectionForGitmoji(gitmoji) as ReleaseSection,
            "parse-noreply-email": (email) => mapAuthor(binding.gitmojiParseNoreplyEmail(email)),
            "resolve-github-author": (email, authorMapJson) =>
                mapAuthor(binding.gitmojiResolveGithubAuthor(email, authorMapJson)),
            "display-login": (author) => binding.gitmojiDisplayLogin(author),
            "profile-url": (author) => binding.gitmojiProfileUrl(author),
            "avatar-url": (author) => binding.gitmojiAvatarUrl(author),
            "author-mention": (email, authorName, authorMapJson) =>
                binding.gitmojiAuthorMention(email, authorName, authorMapJson),
            "commit-bullet": (body, email, authorName, authorMapJson) =>
                binding.gitmojiCommitBullet(body, email, authorName, authorMapJson),
        },
        git: {
            "discover-root": (startPath) => binding.gitDiscoverRoot(startPath),
            "list-version-tags": (repoRoot) => binding.gitListVersionTags(repoRoot),
            "list-tag-infos": (repoRoot) =>
                binding.gitListTagInfos(repoRoot).map((tag) => ({
                    name: tag.name,
                    "short-hash": tag.shortHash,
                })),
            "format-tag-list": (repoRoot) => binding.gitFormatTagList(repoRoot),
            "resolve-range": (repoRoot, version, fromRef, toRef) => {
                const range = binding.gitResolveRange(repoRoot, version, fromRef, toRef);
                return {
                    version: range.version,
                    "from-ref": range.fromRef,
                    "to-ref": range.toRef,
                };
            },
            "collect-commits": (repoRoot, fromRef, toRef) => binding.gitCollectCommits(repoRoot, fromRef, toRef),
            "detect-github-repo": (repoRoot) => binding.gitDetectGithubRepo(repoRoot) ?? undefined,
            "parse-github-remote": (url) => binding.gitParseGithubRemote(url) ?? undefined,
        },
    };
}

let cached: NiftyNative | undefined;

/** Load the platform-specific Node-API binary from `@doki-land/nifty-<platform>`. */
export function loadNiftyNative(): NiftyNative {
    if (cached) {
        return cached;
    }
    const key = `${process.platform}-${process.arch}`;
    const pkg = PLATFORM_PACKAGES[key];
    if (!pkg) {
        throw new Error(`Unsupported platform for Nifty native bindings: ${key}`);
    }
    const require = createRequire(import.meta.url);
    const binding = require(pkg).default as NativeBinding;
    cached = wrapBinding(binding);
    return cached;
}

export function mapTagInfo(raw: { name: string; "short-hash": string }): TagInfo {
    return { name: raw.name, shortHash: raw["short-hash"] };
}

export function mapRangeInfo(raw: { version: string; "from-ref"?: string; "to-ref": string }): RangeInfo {
    return {
        version: raw.version,
        fromRef: raw["from-ref"],
        toRef: raw["to-ref"],
    };
}

export function mapCommitRecord(raw: CommitRecord): CommitRecord {
    return {
        hash: raw.hash,
        email: raw.email,
        author: raw.author,
        subject: raw.subject,
        body: raw.body,
        gitmoji: raw.gitmoji,
        section: raw.section as ReleaseSection,
    };
}
