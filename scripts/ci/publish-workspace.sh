#!/usr/bin/env bash
# Tag publish: merge matrix native artifacts, attach GitHub Release assets, npm publish.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

node scripts/ci/install-native-artifacts.mjs
bash scripts/ci/upload-release.sh
exec pnpm exec nifty publish --access public "$@"
