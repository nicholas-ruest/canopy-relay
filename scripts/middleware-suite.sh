#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/canopy-relay-env.sh"
mapfile -t packages < <(cargo metadata --locked --offline --format-version 1 | node -e '
let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const m=JSON.parse(s);for(const p of m.packages)if(m.workspace_members.includes(p.id)&&/^cr-(device-trust|middleware|cedar-policy|observability|errors)/.test(p.name))console.log(p.name)})')
((${#packages[@]} >= 3)) || { echo 'expected at least three middleware packages' >&2; exit 1; }
args=(); for package in "${packages[@]}"; do args+=(-p "$package"); done
cargo test --locked "${args[@]}"
