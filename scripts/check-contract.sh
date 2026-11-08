#!/usr/bin/env bash
set -euo pipefail
for path in /api/v1/dashboard /api/v1/observations /api/v1/interventions /api/v1/audit; do
  grep -Fq "$path" openapi.yaml
  grep -Fq "$path" frontend/src/client.ts
done
[[ -x frontend/node_modules/.bin/tsc ]] && npm --prefix frontend run typecheck
printf 'API_CONTRACT_PASS endpoints=dashboard,observations,interventions,audit\n'
