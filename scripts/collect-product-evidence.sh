#!/usr/bin/env bash
set -uo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"
: "${DREAM_MACHINE_EVIDENCE_DIR:?external evidence directory required}"
source scripts/canopy-relay-env.sh
export DREAM_MACHINE_SOURCE_SHA="$(git rev-parse HEAD)"
export DREAM_MACHINE_PRODUCER="${DREAM_MACHINE_PRODUCER:-Codex}"
export DREAM_MACHINE_RUN_ID="${DREAM_MACHINE_RUN_ID:-${GITHUB_RUN_ID:-local-${DREAM_MACHINE_SOURCE_SHA:0:12}}}"
export DREAM_MACHINE_GUIDANCE_SHA="${DREAM_MACHINE_GUIDANCE_SHA:-34cc1f31df20d71ea8a42b819091334a981f43cb}"
mkdir -p "$DREAM_MACHINE_EVIDENCE_DIR"
gate=(node scripts/dream-machine-product-gate.mjs record "$ROOT" "$DREAM_MACHINE_EVIDENCE_DIR")
status=0
record(){ local suite="$1"; shift; "${gate[@]}" "$suite" -- "$@" || status=1; }
record rust bash scripts/rust-suite.sh
# The backend gate requires these exact argv and resolved stdout; never add --no-deps.
record backend cargo metadata --locked --offline --format-version 1
record frontend bash scripts/frontend-suite.sh
record api bash scripts/api-suite.sh
record middleware bash scripts/middleware-suite.sh
record integrations bash scripts/integrations-suite.sh
record benchmark bash scripts/benchmark.sh
record ablations bash scripts/ablations-suite.sh
record containers bash scripts/container-suite.sh
record browser bash scripts/browser-suite.sh
if ((status)); then echo EVIDENCE_COLLECTION_FAILED >&2; exit "$status"; fi
node scripts/build-product-evidence.mjs
printf 'PRODUCT_EVIDENCE_COLLECTED source=%s run=%s\n' "$DREAM_MACHINE_SOURCE_SHA" "$DREAM_MACHINE_RUN_ID"
