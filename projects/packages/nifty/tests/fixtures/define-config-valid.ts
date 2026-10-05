import { defineConfig } from "../../src/config/defineConfig.js";

const objectConfig = defineConfig({ format: { preset: "nifty" } });
objectConfig.format?.preset satisfies "nifty";

const factoryConfig = defineConfig(({ mode }) => ({
    format: { preset: mode === "production" ? "nifty" : "default" },
}));
factoryConfig({ mode: "production", command: "build", layout: {} as never });
