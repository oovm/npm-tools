// Nifty project configuration for the npm-tools monorepo.
import { defineConfig } from "@doki-land/nifty";

export default defineConfig({
    format: {
        preset: "nifty",
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
    trust: {
        npm: {
            repo: "oovm/npm-tools",
            file: "publish-npm.yml",
            environment: "NPM_PUBLISH",
        },
    },
});
