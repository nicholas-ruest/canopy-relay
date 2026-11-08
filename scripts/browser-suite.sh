#!/usr/bin/env bash
set -euo pipefail
: "${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"; cd "$ROOT"
source scripts/canopy-relay-env.sh
export CR_RUN_DIR="$DREAM_MACHINE_EVIDENCE_DIR/runtime"
export CR_PID_DIR="$CR_RUN_DIR/pids"
export CR_DATA="$CR_RUN_DIR/data"
export CANOPY_RELAY_DATA="$CR_DATA"
mkdir -p "$CR_RUN_DIR"
cleanup(){ scripts/stop.sh || true; }; trap cleanup EXIT
scripts/stop.sh || true
scripts/start.sh
(cd frontend && ./node_modules/.bin/playwright test)
scripts/restart-journey.sh
grep 'correlation_id' "$CR_RUN_DIR/backend.log" >"$DREAM_MACHINE_EVIDENCE_DIR/correlation-trace.jsonl"
node - "$DREAM_MACHINE_EVIDENCE_DIR/correlation-trace.jsonl" <<'NODE'
const fs=require('fs');const lines=fs.readFileSync(process.argv[2],'utf8').trim().split('\n').filter(Boolean);
if(lines.length<3)throw new Error('insufficient correlated backend/upstream trace rows');
for(const line of lines){const row=JSON.parse(line);const c=row.correlation_id??row.fields?.correlation_id;if(!c)throw new Error('uncorrelated trace row')}
console.log(`CORRELATION_TRACE_PASS rows=${lines.length}`);
NODE
trace="$(find "$DREAM_MACHINE_EVIDENCE_DIR/playwright-results" -type f -name trace.zip | sort | head -1)"
test -n "$trace"; cp "$trace" "$DREAM_MACHINE_EVIDENCE_DIR/browser-trace.zip"
for artifact in browser-report.json desktop.png mobile.png browser-trace.zip correlation-trace.jsonl; do
  test -s "$DREAM_MACHINE_EVIDENCE_DIR/$artifact"
done
printf 'BROWSER_EVIDENCE_PASS\n'
