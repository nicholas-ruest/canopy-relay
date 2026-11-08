# Architecture decision records

Each accepted decision is specific to Canopy Relay, names its production owners, and maps to a DDD source. Execution-dependent consequences remain subject to exact-source verification.

| ADR | Decision | Owning crates | DDD |
|---|---|---|---|
| [0001](0001-composition-root-and-application-services.md) | composition root and use-case boundary | service, application | 01 |
| [0002](0002-typed-errors-and-validated-config.md) | typed failures and bounded config | errors, service, api | 11 |
| [0003](0003-rucelium-signed-anti-replay-ingest.md) | signed evidence and one-use receipts | rucelium, device-trust, application | 02 |
| [0004](0004-calibration-before-twin-mutation.md) | bounded calibration before persistence | calibration, application | 03 |
| [0005](0005-worldgraph-versioned-habitat-provenance.md) | serialized WorldGraph habitat cell | worldgraph, upstream-compose | 04 |
| [0006](0006-midstream-seasonal-anomaly-windows.md) | Midstream DTW sequences | midstream, upstream-compose | 05 |
| [0007](0007-latentmesh-offline-cited-envelopes.md) | LatentMesh constrained route transport | latentmesh, upstream-compose | 06 |
| [0008](0008-ruv-fann-bounded-forecast-abstention.md) | deterministic bounded ruv-FANN forecast | ruvfann, upstream-compose | 07 |
| [0009](0009-explainable-ecological-objective.md) | named five-score composition | upstream-compose, application | 08 |
| [0010](0010-reversible-constraint-first-interventions.md) | reject irreversible action classes | application, contracts | 09 |
| [0011](0011-cedar-denial-before-effects.md) | Cedar denial before effects | policy, application | 09 |
| [0012](0012-middleware-enforces-route-invariants.md) | executable route controls | middleware, api, service | 11 |
| [0013](0013-opendal-authorized-persistence.md) | namespaced OpenDAL boundary | opendal-store, repository | 10 |
| [0014](0014-hash-linked-audit-and-rebuild.md) | audit verification on restore | repository, dashboard | 10 |
| [0015](0015-versioned-openapi-and-checked-client.md) | versioned contract/client | contract, api | 11 |
| [0016](0016-correlation-timeout-and-observability.md) | correlation and request timeout | middleware, application | 11 |
| [0017](0017-production-reachability-of-product-crates.md) | real backend graph | service, application | 01 |
| [0018](0018-three-real-browser-journeys.md) | three browser journeys | api, application, frontend | 12 |
| [0019](0019-sensitive-location-projections.md) | synthetic coarse geometry boundary | seed, dashboard, contract | 02 |
| [0020](0020-deterministic-watershed-seed.md) | deterministic seed/reset | seed | 04 |
| [0021](0021-combined-benchmark-and-ablations.md) | combined baseline and capability-loss ablations | application, upstream-compose | 08 |
| [0022](0022-external-exact-source-evidence.md) | external exact-source proof | verification | 10 |
| [0023](0023-container-parity-and-cloudrun-config-only.md) | container parity, config-only deploy | service/delivery | 11 |
| [0024](0024-accessible-responsive-field-console.md) | accessible product states | frontend, contract | 12 |
| [0025](0025-human-authority-and-explicit-limitations.md) | human authority and limits | application, cedar-policy, README | 12 |

