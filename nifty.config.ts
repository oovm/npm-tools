// Nifty project configuration for the npm-tools monorepo.
import { defineConfig } from "@doki-land/nifty";

export default defineConfig({
    format: {
        preset: "npm-tools",
        style: {
            indentStyle: "space",
            indentWidth: 4,
            lineWidth: 144,
            quoteStyle: "single",
        },
    },
    changelog: {
        repo: "oovm/npm-tools",
        releasesDir: "documentation/maintenance/releases",
    },
    publish: {
        packages: [
            "@doki-land/nifty",
            "@doki-land/nifty-skills",
            "@doki-land/nifty-win32-x64",
            "@doki-land/nifty-linux-x64",
            "@doki-land/nifty-linux-arm64",
            "@doki-land/nifty-darwin-arm64",
            "@doki-land/nifty-darwin-x64",
        ],
    },
});
