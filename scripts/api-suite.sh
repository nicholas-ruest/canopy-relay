#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/canopy-relay-env.sh"
scripts/check-contract.sh
mapfile -t packages < <(cargo metadata --locked --offline --format-version 1 | node -e '
let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const m=JSON.parse(s);for(const p of m.packages)if(m.workspace_members.includes(p.id)&&/^cr-(api|service|application|orchestrator|contracts|types)$/.test(p.name))console.log(p.name)})')
((${#packages[@]} >= 1)) || { echo 'missing Canopy Relay API/service package' >&2; exit 1; }
args=(); for package in "${packages[@]}"; do args+=(-p "$package"); done
cargo test --locked "${args[@]}"
