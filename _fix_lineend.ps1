# 关掉 autocrlf，让 Windows 上 working tree 跟 CI 一样是 LF
git config core.autocrlf false
git config core.eol lf

# 重新 checkout working tree，强制用 blob 里的 LF 版本
git rm --cached -r .
git reset --hard

# 现在 working tree 应该是 LF 了，跑 nifty format 看它要什么
pnpm run build:napi 2>&1 | Select-Object -Last 2
pnpm format --check 2>&1 | Select-Object -Last 20
