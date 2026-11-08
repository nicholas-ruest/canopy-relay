#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/canopy-relay-env.sh"
OUT_DIR="${DREAM_MACHINE_EVIDENCE_DIR:-/tmp/canopy-relay-benchmark}"
mkdir -p "$OUT_DIR"
package="$(cargo metadata --locked --offline --format-version 1 | node -e '
let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{const m=JSON.parse(s);const ws=m.packages.filter(p=>m.workspace_members.includes(p.id));const p=ws.find(p=>/^cr-(evaluation|benchmark)$/.test(p.name))??ws.find(p=>p.targets.some(t=>t.kind.includes("example")&&t.name==="ablation"));if(!p)process.exit(2);process.stdout.write(p.name)})')"
cargo run --locked -p "$package" --example ablation >"$OUT_DIR/benchmark.json"
node - "$OUT_DIR/benchmark.json" <<'NODE'
const fs=require('fs'); const rows=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));
if(!Array.isArray(rows)||rows.length<4) throw new Error('benchmark needs combined, baselines, and ablations');
const combined=rows.find(x=>x.mode==='combined');
if(!combined||combined.success===false) throw new Error('combined system did not succeed');
if(!rows.some(x=>/ruvnet.only|ruvnet_only/.test(x.mode))||!rows.some(x=>/enterprise.only|enterprise_only/.test(x.mode))) throw new Error('missing ingredient baselines');
console.log(`BENCHMARK_PASS rows=${rows.length}`);
NODE
