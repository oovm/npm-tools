#!/usr/bin/env bash
# Tag publish: merge matrix native artifacts into platform packages, then npm publish (OIDC).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

node scripts/ci/install-native-artifacts.mjs
exec pnpm exec nifty publish --access public "$@"
