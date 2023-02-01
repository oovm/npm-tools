# @doki-land/nifty-linter

Rule-based lint/check for Nifty gitmoji commit conventions.

## CLI

```bash
nifty lint
nifty lint --subject "✨ Add feature"
nifty check
```

## Programmatic

```ts
import { lintSubjects } from "@doki-land/nifty-linter";

const report = lintSubjects(["✨ Add feature", "Add bad subject"]);
console.log(report.errorCount);
```

Default rules: `gitmoji/subject`, `gitmoji/known`, `gitmoji/body`, `gitmoji/format`.
