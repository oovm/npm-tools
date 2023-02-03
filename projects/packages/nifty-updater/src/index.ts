import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { detectProjectLayout } from "@doki-land/nifty-config";

export type UpdateOptions = {
    /** Prompt before applying each ecosystem's updates. */
    interactive?: boolean;
    /** Directory to detect the project layout from. */
    cwd?: string;
};

const crateManifest = join(
    dirname(fileURLToPath(import.meta.url)),
    "../../../crates/nifty-updater/Cargo.toml",
);

/** Run `nifty update` via the Rust CLI (`cargo upgrade` + `npm update`). */
export async function update(options: UpdateOptions = {}): Promise<void> {
    const cwd = options.cwd ?? process.cwd();
    const layout = detectProjectLayout(cwd);
    if (layout.kind === "unknown") {
        throw new Error(`no Cargo.toml or package.json found from ${cwd}`);
    }

    const args = ["run", "--quiet", "--manifest-path", crateManifest, "--", "update"];
    if (options.interactive) {
        args.push("-i");
    }
    args.push("-C", cwd);

    const result = spawnSync("cargo", args, {
        stdio: "inherit",
        cwd,
    });

    if (result.status !== 0) {
        throw new Error("nifty update failed");
    }
}

export { detectProjectLayout };
