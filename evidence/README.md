# Canopy Relay runtime evidence

`scripts/verify-prototype.sh` records all evidence outside the Git checkout and binds every receipt and artifact to the exact clean candidate SHA. CI uploads the directory even when a suite fails, but the canonical gate itself is never allowed to continue on error.

The durable artifact includes the schema-2 manifest and composition report, original schema-1 selection report and readiness archive, exact `cargo metadata --locked --offline --format-version 1` stdout and separately hashed stderr, ten suite receipts/logs, Rust-share audit, semantic inventory, benchmark/ablations, container proof, Playwright JSON/trace, desktop/mobile PNGs, and correlated UI/backend/upstream trace.

The canonical structural result is necessary evidence only. Final review must still inspect crate substance, function call paths, upstream execution, middleware denial-before-effects, restart-durable retrieval, UI quality, and producer/run attribution before claiming `BUILT`.
