# @doki-land/nifty-darwin-arm64

Prebuilt Node-API native artifact for **macOS Apple Silicon (arm64)**.

**Not a public import target.** `@doki-land/nifty` pulls this package automatically on matching hosts via optional dependencies.

## Example

Application code imports the main package only:

```ts
import { createNifty } from "@doki-land/nifty";

const nifty = await createNifty({ cwd: process.cwd() });
nifty.git.listVersionTags(nifty.layout.root);
```

See [@doki-land/nifty](https://www.npmjs.com/package/@doki-land/nifty) · [Source](https://github.com/oovm/npm-tools/tree/dev/projects/packages/nifty-darwin-arm64)
