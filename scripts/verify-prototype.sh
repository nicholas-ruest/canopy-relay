#!/usr/bin/env bash
# Full-product verification. Slice diagnostics live in verify-slice.sh.
set -euo pipefail
task_source_root="$(git rev-parse --show-toplevel)"
cd "$task_source_root"
node scripts/dream-machine-gate-bundle.mjs .
task_source_sha="$(git rev-parse HEAD)"
if [[ -n "$(git status --porcelain --untracked-files=all)" ]]; then
  echo 'NONCOMPLIANT_BUILD: source must be clean before evidence collection' >&2
  exit 1
fi
task_evidence_dir="${DREAM_MACHINE_EVIDENCE_DIR:-$(mktemp -d)}"
mkdir -p "$task_evidence_dir"
task_evidence_dir="$(realpath "$task_evidence_dir")"
case "$task_evidence_dir" in
  "$task_source_root"|"$task_source_root"/*)
    echo 'NONCOMPLIANT_BUILD: evidence must be outside the source checkout' >&2
    exit 1
    ;;
esac
export DREAM_MACHINE_EVIDENCE_DIR="$task_evidence_dir"
export DREAM_MACHINE_SOURCE_SHA="$task_source_sha"
task_collection_status=0
if [[ -f scripts/collect-product-evidence.sh ]]; then
  bash scripts/collect-product-evidence.sh || task_collection_status=$?
else
  echo 'NONCOMPLIANT_BUILD: missing real whole-system evidence collector' >&2
  task_collection_status=1
fi
# Always execute the canonical product verifier, including on incomplete builds.
task_gate_status=0
node scripts/dream-machine-product-gate.mjs verify . "$task_evidence_dir" "$task_source_sha" || task_gate_status=$?
if (( task_collection_status != 0 || task_gate_status != 0 )); then
  echo 'NONCOMPLIANT_BUILD: full product verification failed; slice checks cannot override it' >&2
  exit 1
fi
