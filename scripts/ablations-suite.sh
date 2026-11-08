#!/usr/bin/env bash
set -euo pipefail
scripts/benchmark.sh
node - "${DREAM_MACHINE_EVIDENCE_DIR:-/tmp/canopy-relay-benchmark}/benchmark.json" <<'NODE'
const fs=require('fs');const rows=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));
const ablations=rows.filter(x=>/^without[-_]/.test(x.mode));
if(ablations.length<7) throw new Error(`expected >=7 ingredient ablations, got ${ablations.length}`);
if(ablations.some(x=>typeof x.mode!=='string')) throw new Error('invalid ablation row');
console.log(`ABLATIONS_PASS count=${ablations.length}`);
NODE
