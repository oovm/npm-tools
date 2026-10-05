import { defineConfig } from "../../src/config/defineConfig.js";

defineConfig({
    format: { preset: "nifty" },
    unknownOption: {},
});

defineConfig(({ mode }) => ({
    format: { preset: mode === "production" ? "nifty" : "default" },
}));
