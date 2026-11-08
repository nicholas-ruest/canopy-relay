#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/canopy-relay-env.sh"
node scripts/dream-machine-gate-bundle.mjs .
node scripts/dream-machine-rust-share.mjs --self-test
node scripts/dream-machine-rust-share.mjs . >/tmp/canopy-relay-rust-share.json
scripts/rust-suite.sh
scripts/frontend-suite.sh
scripts/api-suite.sh
scripts/middleware-suite.sh
scripts/start.sh
trap 'scripts/stop.sh' EXIT
curl -fsS "$CR_BASE_URL/readyz"
curl -fsS "$CR_FRONTEND_URL/" | grep -q 'id="root"'
curl -fsS -H "Authorization: Bearer $CR_TOKEN" "$CR_BASE_URL/api/v1/dashboard" >/dev/null
printf 'WORKING_PRODUCT_SLICE_PASS\n'
