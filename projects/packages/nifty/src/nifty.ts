import {

    authorMapToJson,

    detectProjectLayout,

    loadConfig,

    type LoadConfigOptions,

    type NiftyConfig,

    type ProjectLayout,

} from "@doki-land/nifty-config";



import { Git } from "./git.js";

import { Github } from "./github.js";

import { Gitmoji } from "./gitmoji.js";

import { loadNiftyWasm } from "./wasm.js";



export type NiftyOpenOptions = NiftyConfig &

    LoadConfigOptions & {

        /** When `false`, skip loading `nifty.config.ts/js`. Inline options still apply. */

        config?: false;

    };



function resolveRoots(config: NiftyConfig, layout: ProjectLayout): NiftyConfig {

    return {

        ...config,

        repoRoot: config.repoRoot ?? layout.root,

        cargoRoot:

            config.cargoRoot ??

            layout.cargoWorkspaceRoot ??

            (layout.cargoManifest ? layout.cargoManifest.replace(/[/\\]Cargo\.toml$/, "") : undefined),

        npmRoot:

            config.npmRoot ??

            layout.npmWorkspaceRoot ??

            (layout.packageManifest ? layout.packageManifest.replace(/[/\\]package\.json$/, "") : undefined),

    };

}



/** Combined Nifty engine: gitmoji conventions + gix repository reads + GitHub API. */

export class Nifty {

    readonly gitmoji: Gitmoji;

    readonly git: Git;

    readonly github: Github;

    readonly config: NiftyConfig;

    readonly layout: ProjectLayout;

    readonly configFile?: string;



    private constructor(

        gitmoji: Gitmoji,

        git: Git,

        github: Github,

        config: NiftyConfig,

        layout: ProjectLayout,

        configFile?: string,

    ) {

        this.gitmoji = gitmoji;

        this.git = git;

        this.github = github;

        this.config = config;

        this.layout = layout;

        this.configFile = configFile;

    }



    static async open(options: NiftyOpenOptions = {}): Promise<Nifty> {

        const cwd = options.cwd ?? process.cwd();

        const layout = detectProjectLayout(cwd);

        const { config: skipConfig, configFile, env, githubToken, authorMap, repoRoot, cargoRoot, npmRoot } = options;

        const loaded =

            skipConfig === false

                ? { config: {} as NiftyConfig }

                : await loadConfig({ cwd, configFile, env: { layout, ...env } });

        const merged = resolveRoots(

            {

                ...loaded.config,

                ...(githubToken !== undefined ? { githubToken } : {}),

                ...(authorMap !== undefined ? { authorMap } : {}),

                ...(repoRoot !== undefined ? { repoRoot } : {}),

                ...(cargoRoot !== undefined ? { cargoRoot } : {}),

                ...(npmRoot !== undefined ? { npmRoot } : {}),

            },

            layout,

        );



        const wasm = await loadNiftyWasm();

        return new Nifty(

            new Gitmoji(wasm.gitmoji, authorMapToJson(merged.authorMap)),

            new Git(wasm.git),

            Github.open(merged.githubToken),

            merged,

            layout,

            loaded.configFile,

        );

    }

}



export async function createNifty(options: NiftyOpenOptions = {}): Promise<Nifty> {

    return Nifty.open(options);

}


