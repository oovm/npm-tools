// Nifty project configuration for the npm-tools monorepo.
import { defineConfig } from "@doki-land/nifty";

export default defineConfig({
    format: {
        preset: "npm-tools",
        style: "biome.json",
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
