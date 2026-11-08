# Release governance

Canopy Relay is published only after the isolated local candidate passes the complete Dream Machine product gate and an independent semantic review.

## Source and producer binding

- Toronto run date: 2026-11-07
- Producer: Codex
- Producer run: `dm-20261107-canopy-relay`
- Effective guidance: `nicholas-ruest/core-memory@34cc1f31df20d71ea8a42b819091334a981f43cb`
- Product/selection/backend gate SHA-256: `193079f8…a0e`, `27e99d82…ef49`, `82fa38c2…1e`
- Rust-share helper: `nicholas-ruest/core-memory@e66c5d32ee48491af31f050a8902c3cfa793f09f`, SHA-256 `ba06d9af…d801`

## Promotion rules

The non-draft release PR must run the separate Rust-source-share step before the unconditional canonical full-product verifier. The workflow must test its exact head and upload the external evidence directory even on failure. A green integrity, smoke, or slice check is not a release signal.

Release review reads the actual artifact and checks: 22 independently accepted production crates (with seven repetitive support crates excluded), 25 ADRs, 12 DDD documents, five executed Ruvnet projects, two executed enterprise projects, three real browser journeys, restart-durable retrieval, source hygiene, Rust source share at or above 50%, and truthful limitations. The PR is not merged and no deployment or spend is initiated without separate authority.
