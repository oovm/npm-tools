import { defineConfig } from "../../src/config/defineConfig.js";

defineConfig({ format: { preset: "nifty" } });

defineConfig(({ mode }) => ({
    format: { preset: mode === "production" ? "nifty" : "default" },
}));
