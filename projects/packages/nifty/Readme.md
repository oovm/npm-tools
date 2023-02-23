# @doki-land/nifty

**Gitmoji is the default Nifty convention.**

| Layer | Role |
| --- | --- |
| `@doki-land/nifty-<platform>` | Node-API `.node`（gitmoji + gix） |
| `nifty-publisher` | Rust：`petgraph` 拓扑排序 + `npm publish` |
| `src/cli/` | CLI 实现（`update` / `lint` / `check` / `upload` / `bump` / `publish`） |
| `cli/nifty.mjs` | npm `bin` shim（jiti 加载 `src/cli`） |
| `src/` | TypeScript API |

Rust 侧无 `bin` / `cli`。

**本仓开发**（pnpm workspace，`projects/packages/*`）：

```bash
pnpm install          # 链接 @doki-land/nifty → node_modules/.bin/nifty
pnpm run build:napi   # 构建 native addon
pnpm exec nifty bump
pnpm exec nifty publish --dry-run
```

**发布后消费**：

```bash
npm install @doki-land/nifty
nifty bump
nifty publish --dry-run
```
