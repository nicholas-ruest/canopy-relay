# Core-memory guidance map

| Obligation | Canopy Relay location | Verification |
|---|---|---|
| full product, not CLI/scaffold | Rust service, React, API, middleware, storage, journeys | launched browser product and semantic review |
| first vertical slice first | browser-to-OpenDAL primary workflow | milestone correlation trace |
| >=20 meaningful crates | 22 independently counted; 29 production-reachable | Cargo graph plus independent inspection |
| >=25 ADRs / >=12 DDDs | `docs/adr`, `docs/ddd` | unique source hashes |
| >=5 Ruvnet / >=2 enterprise | integration matrix | selection/probe equality and runtime evidence |
| attempted-build rotation | selection receipt/report | gate plus history review |
| typed API/client | `cr-contracts`, frontend client | drift test |
| executable middleware | `cr-middleware` and owners | order/failure tests |
| authorized persistence | `cr-opendal-store`, `cr-repository` | write/read/restart and audit verification |
| exact backend metadata | recorder and locked offline command | stdout/stderr hashes and backend gate |
| source-bound evidence | external schema-v2 artifact | exact-source readback |
| no deployment/spend | configuration only | repository readback |

Effective revisions and gate hashes belong in `core-memory-receipt.md`. This map does not convert an obligation into a PASS.

## Lesson application map

| Screening class | Source / record | Decision | Code or test | Evidence |
|---|---|---|---|---|
| verified repository guidance | `core-memory@34cc1f3…` completion-first and product-gate sections | local-first full product; no remote before full local PASS | `scripts/verify-prototype.sh`, all ten suite runners | external manifest and gate output |
| verified repository guidance | `core-memory@e66c5d3…` November 7 supplement | require genuine Rust share of at least 50% | exact `scripts/dream-machine-rust-share.mjs` | `rust-share.json` plus CI step |
| source-pinned example | `mandate-mesh@fa6e7ce…` | retain full browser/API/middleware/backend/persistence product shape while rotating the domain and ingredients | architecture, React navigation, service composition | browser traces and semantic review |
| unscreened historical recall | dual-namespace records sourced by `3edd426…`, `d38e093…`, `1559c744…`, `04576aaf…`, `58573c75…`, `a507f788…`, `d1e5f09b…` | no normative decision; retained only as historical evidence after deduplication | none | recall receipt only |
| verified-approved learned lesson | `NONE_AVAILABLE` | no additional learned-policy decision | none | dual-namespace recall counts |

The injection-screening `unscreened:true` flag is reported separately from the empty `dream-machine` result. It did not cause records from the second namespace to be dropped, and retrieval did not confer approval.

