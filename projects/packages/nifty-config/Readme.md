# @doki-land/nifty-config

Load and type Nifty project configuration from `nifty.config.ts` / `nifty.config.js`.

## Usage

```ts
// nifty.config.ts
import { defineConfig } from "@doki-land/nifty-config";

export default defineConfig({
  githubToken: process.env.GITHUB_TOKEN,
  authorMap: {
    "you@example.com": { login: "octocat" },
  },
});
```

```ts
import { loadConfig } from "@doki-land/nifty-config";

const { config, configFile } = await loadConfig();
console.log(configFile, config.githubToken);
```

Functional configs are supported:

```ts
export default defineConfig(({ mode, layout }) => ({
  githubToken: mode === "production" ? process.env.GITHUB_TOKEN : undefined,
  repoRoot: layout.root,
  cargoRoot: layout.cargoWorkspaceRoot,
}));
```

## Project detection

Nifty supports cargo + npm hybrid monorepos. `detectProjectLayout(cwd)` walks upward and returns:

- `kind`: `"cargo" | "npm" | "hybrid" | "unknown"`
- `root`: git root when present
- `cargoManifest`, `packageManifest`
- `cargoWorkspaceRoot`, `npmWorkspaceRoot`

```ts
import { detectProjectLayout } from "@doki-land/nifty-config";

const layout = detectProjectLayout(process.cwd());
if (layout.kind === "hybrid") {
  console.log(layout.cargoWorkspaceRoot, layout.packageManifest);
}
```

## Config files

Searched from the project root upward, first match wins:

- `nifty.config.ts`
- `nifty.config.js`
- `nifty.config.mjs`
- `nifty.config.cjs`
