#!/usr/bin/env bash
# Tag publish: stage matrix native artifacts, then npm publish (OIDC).
# Uses published @doki-land/nifty when available (see scripts/ci/nifty.mjs).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

node scripts/ci/nifty.mjs install-native --from "${NATIVE_ARTIFACT_ROOT:-native-artifacts}"
exec node scripts/ci/nifty.mjs publish --access public "$@"
