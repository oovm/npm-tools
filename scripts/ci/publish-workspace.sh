#!/usr/bin/env bash
# Tag publish: npm OIDC via scripts/ci/publish-npm.mjs (vmz-framework pattern).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

exec node scripts/ci/publish-npm.mjs "$@"
