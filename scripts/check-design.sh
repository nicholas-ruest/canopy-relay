#!/usr/bin/env bash
set -euo pipefail
crates="$(find crates -mindepth 2 -maxdepth 2 -name Cargo.toml -path '*/cr-*/*' | wc -l)"
adrs="$(find docs/adr -maxdepth 1 -name '[0-9][0-9][0-9][0-9]-*.md' | wc -l)"
ddds="$(find docs/ddd -maxdepth 1 -name '[0-9][0-9]-*.md' | wc -l)"
test "$crates" -ge 20
test "$adrs" -ge 25
test "$ddds" -ge 12
test -s docs/adr/index.md
test -s docs/ddd/index.md
test -s docs/crate-to-backend-call-path.md
printf 'STRUCTURAL_INVENTORY_PASS crates=%s adrs=%s ddds=%s (semantic review still required)\n' "$crates" "$adrs" "$ddds"
