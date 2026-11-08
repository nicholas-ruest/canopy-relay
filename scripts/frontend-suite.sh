#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/canopy-relay-env.sh"
[[ -d frontend/node_modules ]] || npm --prefix frontend ci
npm --prefix frontend run typecheck
npm --prefix frontend run lint
npm --prefix frontend run test
npm --prefix frontend run build
