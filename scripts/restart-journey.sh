#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"; cd "$ROOT"
source scripts/canopy-relay-env.sh
auth=(-H "Authorization: Bearer $CR_TOKEN")
body='{"anomaly_id":"anomaly-mesh-loss","action":"deploy-mesh-relay","risk_level":"guarded","rationale":"Restore safe habitat telemetry delivery after the degraded relay was verified."}'
response="$(curl -fsS "${auth[@]}" -H 'Content-Type: application/json' -d "$body" "$CR_BASE_URL/api/v1/interventions")"
id="$(printf '%s' "$response" | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const x=JSON.parse(s);const id=x.data?.id??x.id;if(!id)process.exit(2);console.log(id)})')"
pid="$(<"$CR_PID_DIR/backend.pid")"; kill "$pid"
for _ in $(seq 1 40); do kill -0 "$pid" 2>/dev/null || break; sleep 0.25; done
CANOPY_RELAY_ADDR="$CANOPY_RELAY_ADDR" CANOPY_RELAY_DATA="$CANOPY_RELAY_DATA" RUST_LOG="${RUST_LOG:-info}" \
  cargo run --locked -p "$CR_SERVICE_PACKAGE" >>"$CR_RUN_DIR/backend.log" 2>&1 &
printf '%s\n' "$!" >"$CR_PID_DIR/backend.pid"
for _ in $(seq 1 120); do curl -fsS "$CR_BASE_URL/readyz" >/dev/null 2>&1 && break; sleep 1; done
curl -fsS "${auth[@]}" "$CR_BASE_URL/api/v1/interventions" | node -e '
let s="";const id=process.argv[1];process.stdin.on("data",d=>s+=d).on("end",()=>{const x=JSON.parse(s);const a=x.data??[];if(!a.some(v=>v.id===id))process.exit(1);console.log(`RESTART_RETRIEVAL_PASS ${id}`)})' "$id"
