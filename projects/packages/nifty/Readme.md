# @doki-land/nifty

**Gitmoji is the default Nifty convention.**

| Layer | Role |
| --- | --- |
| `@doki-land/nifty-<platform>` | Node-API `.node`（gitmoji + gix） |
| `src/cli/` | CLI 实现（`update` / `lint` / `check` / `upload` / `bump` / `publish`） |
| `bin/nifty.mjs` | npm `bin` shim（jiti 加载 `src/cli`） |
| `src/` | TypeScript API |

Rust 侧无 `bin` / `cli`。安装后：

```bash
npm install @doki-land/nifty
npm run build:napi   # 开发本仓时构建 native addon
nifty bump              # 全仓版本对齐（默认 0.0.0）
nifty publish --dry-run # 按依赖拓扑顺序发布 npm 包
nifty update
nifty lint
nifty upload --release
```
