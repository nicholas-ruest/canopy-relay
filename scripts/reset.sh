#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"; cd "$ROOT"
source scripts/canopy-relay-env.sh
scripts/stop.sh
test "$CR_RUN_DIR" != / && test -n "$CR_RUN_DIR"
rm -rf "$CR_RUN_DIR" frontend/test-results frontend/playwright-report
mkdir -p "$CR_RUN_DIR" "$CR_PID_DIR"
printf 'CANOPY_RELAY_RESET run_dir=%s\n' "$CR_RUN_DIR"
