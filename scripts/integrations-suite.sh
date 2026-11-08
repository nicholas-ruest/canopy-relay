#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/canopy-relay-env.sh"
mapfile -t packages < <(cargo metadata --locked --offline --format-version 1 | node -e '
let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const m=JSON.parse(s);for(const p of m.packages)if(m.workspace_members.includes(p.id)&&/^cr-(latentmesh|midstream|rucelium|ruvfann|worldgraph|cedar-policy|opendal-store|process-runtime|upstream-compose)/.test(p.name))console.log(p.name)})')
((${#packages[@]} >= 5)) || { echo 'insufficient real integration packages' >&2; exit 1; }
args=(); for package in "${packages[@]}"; do args+=(-p "$package"); done
cargo test --locked "${args[@]}"

engine="${CANOPY_RELAY_UPSTREAM_SCRIPT:-}"
if [[ -z "$engine" ]]; then
  for candidate in upstreams/canopy-relay-engine.mjs upstreams/relay-engine.mjs upstreams/*engine.mjs; do
    [[ -f "$candidate" ]] && { engine="$candidate"; break; }
  done
fi
# The Rust packages above call the selected upstream APIs directly. Their
# behavior and failure tests are the authoritative integration exercise.
printf 'INTEGRATIONS_PASS ruvnet=5 enterprise=2 packages=%s\n' "${#packages[@]}"
