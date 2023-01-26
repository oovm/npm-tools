import type { CommitRecord, GithubAuthor, ParsedSubject, RangeInfo, ReleaseSection, TagInfo } from "./types.js";
import { loadNiftyWasm, mapCommitRecord, mapRangeInfo, mapTagInfo } from "./wasm.js";

/**
 * Nifty default convention: every commit subject starts with a gitmoji followed by a space.
 */
export class Gitmoji {
    constructor(private readonly wasm: Awaited<ReturnType<typeof loadNiftyWasm>>["gitmoji"]) {}

    static async open(): Promise<Gitmoji> {
        const wasm = await loadNiftyWasm();
        return new Gitmoji(wasm.gitmoji);
    }

    knownGitmojis(): string[] {
        return this.wasm["known-gitmojis"]();
    }

    validateSubject(subject: string): boolean {
        return this.wasm["validate-subject"](subject);
    }

    parseSubject(subject: string): ParsedSubject {
        return this.wasm["parse-subject"](subject);
    }

    formatSubject(gitmoji: string, body: string): string {
        return this.wasm["format-subject"](gitmoji, body);
    }

    stripGitmoji(subject: string): string {
        return this.wasm["strip-gitmoji"](subject);
    }

    leadingGitmoji(subject: string): string | undefined {
        return this.wasm["leading-gitmoji"](subject);
    }

    sectionForGitmoji(gitmoji?: string): ReleaseSection {
        return this.wasm["section-for-gitmoji"](gitmoji);
    }

    parseNoreplyEmail(email: string): GithubAuthor | undefined {
        return this.wasm["parse-noreply-email"](email);
    }

    resolveGithubAuthor(email: string, authorMapJson = "{}"): GithubAuthor | undefined {
        return this.wasm["resolve-github-author"](email, authorMapJson);
    }

    displayLogin(author: GithubAuthor): string {
        return this.wasm["display-login"](author);
    }

    profileUrl(author: GithubAuthor): string {
        return this.wasm["profile-url"](author);
    }

    avatarUrl(author: GithubAuthor): string {
        return this.wasm["avatar-url"](author);
    }

    authorMention(email: string, authorName: string, authorMapJson = "{}"): string {
        return this.wasm["author-mention"](email, authorName, authorMapJson);
    }

    commitBullet(body: string, email: string, authorName: string, authorMapJson = "{}"): string {
        return this.wasm["commit-bullet"](body, email, authorName, authorMapJson);
    }
}

export async function createGitmoji(): Promise<Gitmoji> {
    return Gitmoji.open();
}
