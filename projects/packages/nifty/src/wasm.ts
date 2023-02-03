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

export type GithubExports = {
    "user-by-login": (
        login: string,
        token: string | undefined,
    ) => { tag: "ok"; val: GithubAuthor } | { tag: "err"; val: string };
    "search-user-by-email": (
        email: string,
        token: string,
    ) => { tag: "ok"; val: GithubAuthor | undefined } | { tag: "err"; val: string };
    "lookup-user-by-email": (
        email: string,
        authorMapJson: string,
        token: string | undefined,
        fetch: boolean,
    ) => { tag: "ok"; val: GithubAuthor | undefined } | { tag: "err"; val: string };
};

export type NiftyWasm = {
    gitmoji: GitmojiExports;
    git: GitExports;
    github?: GithubExports;
};

export function mapGithubAuthor(raw: { id?: bigint; login?: string }): GithubAuthor {
    return { id: raw.id, login: raw.login };
}

let cached: NiftyWasm | undefined;

/** Load the WASI component compiled into `lib/` (via `jco transpile`). */
export async function loadNiftyWasm(): Promise<NiftyWasm> {
    if (cached) {
        return cached;
    }
    const { default: instantiate } = await import("../lib/nifty.js");
    const { exports } = await instantiate();
    cached = exports as NiftyWasm;
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

export function mapCommitRecord(raw: {
    hash: string;
    email: string;
    author: string;
    subject: string;
    body: string;
    gitmoji?: string;
    section: string;
}): CommitRecord {
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
