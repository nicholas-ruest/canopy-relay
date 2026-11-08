#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"; cd "$ROOT"
source scripts/canopy-relay-env.sh
mkdir -p "$CR_RUN_DIR" "$CR_PID_DIR" "$(dirname "$CR_DATA")"
[[ -x frontend/node_modules/.bin/vite ]] || npm --prefix frontend ci
if [[ -f upstreams/package-lock.json && ! -d upstreams/node_modules ]]; then
  npm --prefix upstreams ci --ignore-scripts --no-audit --no-fund
fi
scripts/stop.sh
CANOPY_RELAY_ADDR="$CANOPY_RELAY_ADDR" CANOPY_RELAY_DATA="$CANOPY_RELAY_DATA" RUST_LOG="${RUST_LOG:-info}" \
  cargo run --locked -p "$CR_SERVICE_PACKAGE" >"$CR_RUN_DIR/backend.log" 2>&1 &
printf '%s\n' "$!" >"$CR_PID_DIR/backend.pid"
(cd frontend && exec ./node_modules/.bin/vite --host 127.0.0.1 --port 5173) \
  >"$CR_RUN_DIR/frontend.log" 2>&1 &
printf '%s\n' "$!" >"$CR_PID_DIR/frontend.pid"
for _ in $(seq 1 120); do
  if curl -fsS "$CR_BASE_URL/readyz" >/dev/null 2>&1 && curl -fsS "$CR_FRONTEND_URL/" >/dev/null 2>&1; then
    printf 'CANOPY_RELAY_READY api=%s frontend=%s data=%s\n' "$CR_BASE_URL" "$CR_FRONTEND_URL" "$CR_DATA"
    exit 0
  fi
  sleep 1
done
tail -100 "$CR_RUN_DIR/backend.log" >&2 || true
tail -100 "$CR_RUN_DIR/frontend.log" >&2 || true
echo 'Canopy Relay readiness timed out' >&2; exit 1
