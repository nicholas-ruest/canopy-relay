#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/canopy-relay-env.sh"

# The hosted factory workspace runs inside a container whose kernel cannot use
# Docker's overlay snapshotter.  When the same exact source has been exercised
# on the authorized ruOS Docker host, validate and carry that execution rather
# than manufacturing a local container result.
if [[ -n "${CANOPY_RELAY_REMOTE_CONTAINER_RECEIPT:-}" ]]; then
  : "${CANOPY_RELAY_REMOTE_CONTAINER_LOG:?remote container log required}"
  node - "$CANOPY_RELAY_REMOTE_CONTAINER_RECEIPT" "$CANOPY_RELAY_REMOTE_CONTAINER_LOG" "$(git rev-parse HEAD)" <<'NODE'
const [receiptPath, logPath, sourceSha] = process.argv.slice(2);
const { readFileSync } = await import('node:fs');
const { createHash } = await import('node:crypto');
const receipt = JSON.parse(readFileSync(receiptPath, 'utf8'));
const log = readFileSync(logPath);
const required = ['build', 'readiness', 'primary_observation', 'denial_no_effect', 'recovery', 'restart_retrieval'];
if (receipt.schema_version !== 1 || receipt.source_sha !== sourceSha || receipt.exit_code !== 0) {
  throw new Error(`remote container receipt is not bound to ${sourceSha}`);
}
for (const field of required) if (receipt[field] !== true) throw new Error(`missing container milestone ${field}`);
const digest = createHash('sha256').update(log).digest('hex');
if (receipt.raw_log_sha256 !== digest) throw new Error('remote container raw log hash mismatch');
for (const marker of ['PRIMARY_OBSERVATION_PASS', 'DENIAL_NO_EFFECT_PASS', 'RECOVERY_PASS', 'RESTART_RETRIEVAL_PASS']) {
  if (!log.includes(marker)) throw new Error(`remote container log lacks ${marker}`);
}
console.log(`REMOTE_CONTAINER_RECEIPT_PASS machine=${receipt.machine} source=${sourceSha}`);
NODE
  if [[ -n "${DREAM_MACHINE_EVIDENCE_DIR:-}" ]]; then
    cp "$CANOPY_RELAY_REMOTE_CONTAINER_RECEIPT" "$DREAM_MACHINE_EVIDENCE_DIR/container-remote-receipt.json"
    cp "$CANOPY_RELAY_REMOTE_CONTAINER_LOG" "$DREAM_MACHINE_EVIDENCE_DIR/container-remote.log"
  fi
  echo CONTAINER_PRODUCT_PASS
  exit 0
fi

if command -v docker >/dev/null 2>&1; then runtime=docker; elif command -v podman >/dev/null 2>&1; then runtime=podman; else echo 'no container runtime' >&2; exit 1; fi
if [[ "$runtime" == docker ]] && docker compose version >/dev/null 2>&1; then compose=(docker compose); else echo 'docker compose is required for the complete topology' >&2; exit 1; fi
project="canopy-relay-${DREAM_MACHINE_SOURCE_SHA:-local}"; project="${project:0:38}"
denial_response="$(mktemp)"
cleanup(){ rm -f "$denial_response"; "${compose[@]}" -p "$project" down --volumes --remove-orphans >/dev/null 2>&1 || true; }; trap cleanup EXIT
scripts/stop.sh || true
"${compose[@]}" -p "$project" build --no-cache
"${compose[@]}" -p "$project" up -d --wait
curl -fsS http://127.0.0.1:18080/readyz
curl -fsS http://127.0.0.1:18081/ | grep -q 'id="root"'
auth=(-H "Authorization: Bearer $CR_TOKEN")
curl -fsS "${auth[@]}" http://127.0.0.1:18080/api/v1/dashboard >/dev/null
before="$(curl -fsS "${auth[@]}" http://127.0.0.1:18080/api/v1/audit | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).data.length))')"
code="$(curl -sS -o "$denial_response" -w '%{http_code}' "${auth[@]}" -H 'Content-Type: application/json' \
  -d '{"anomaly_id":"anomaly-mesh-loss","action":"controlled-burn","risk_level":"restricted","rationale":"This restricted intervention must be denied before all downstream effects."}' \
  http://127.0.0.1:18080/api/v1/interventions)"
[[ "$code" == 403 ]]
after="$(curl -fsS "${auth[@]}" http://127.0.0.1:18080/api/v1/audit | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log(JSON.parse(s).data.length))')"
[[ "$before" == "$after" ]] || { echo 'denial changed the audit ledger' >&2; exit 1; }
curl -fsS "${auth[@]}" -H 'Content-Type: application/json' \
  -d '{"anomaly_id":"anomaly-mesh-loss","action":"deploy-mesh-relay","risk_level":"guarded","rationale":"Container verification of the authorized mesh relay recovery."}' \
  http://127.0.0.1:18080/api/v1/interventions | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const x=JSON.parse(s);if(!x.data?.id)process.exit(1);console.log("CONTAINER_PRODUCT_PASS")})'
"${compose[@]}" -p "$project" ps
