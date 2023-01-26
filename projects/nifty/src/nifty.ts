import { Git } from "./git.js";
import { Github } from "./github.js";
import { Gitmoji } from "./gitmoji.js";
import { loadNiftyWasm } from "./wasm.js";

export type NiftyOpenOptions = {
    /** GitHub personal access token for email search / profile fetch. */
    githubToken?: string;
};

/** Combined Nifty engine: gitmoji conventions + gix repository reads + GitHub API. */
export class Nifty {
    readonly gitmoji: Gitmoji;
    readonly git: Git;
    readonly github: Github;

    private constructor(gitmoji: Gitmoji, git: Git, github: Github) {
        this.gitmoji = gitmoji;
        this.git = git;
        this.github = github;
    }

    static async open(options: NiftyOpenOptions = {}): Promise<Nifty> {
        const wasm = await loadNiftyWasm();
        return new Nifty(
            new Gitmoji(wasm.gitmoji),
            new Git(wasm.git),
            Github.open(options.githubToken),
        );
    }
}

export async function createNifty(): Promise<Nifty> {
    return Nifty.open();
}
