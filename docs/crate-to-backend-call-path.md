# Crate-to-backend call-path map

The locked production graph starts at the `cr-service` binary. Each row identifies owned behavior, a normal production caller, success/failure proof, and reciprocal design sources. Cargo membership alone is never counted.

Semantic review counts 22 crates with distinct responsibilities. `cr-ingest`, `cr-habitat-graph`, `cr-mesh-transport`, `cr-neural-forecast`, `cr-intervention`, `cr-audit`, and `cr-observability` are production-reachable evidence-book support packages with a deliberately shared record/lineage model; they are exercised but excluded from the meaningful-crate total.

| Crate | Owned typed behavior | Production caller / path | Behavior and failure proof | ADR / DDD |
|---|---|---|---|---|
| `cr-types` | ecological API/domain values and IDs | every domain crate | location/ID tests; invalid bounds | 0001,0002 / 01,12 |
| `cr-errors` | stable failure taxonomy | API, storage, upstream and domain crates | status mapping; redaction/integrity variants | 0002 / 11 |
| `cr-contracts` | versioned envelopes and request invariants | `cr-api`, `cr-application` | roundtrip; invalid observation/intervention | 0015 / 11 |
| `cr-device-trust` | provisioned-device admission, monotonic sequence, one-use receipts | `cr-application::new/observe` | valid admission; unknown/revoked/replayed receipt | 0003 / 02 |
| `cr-ingest` | support ledger for accepted observation metrics (not counted) | `cr-application::observe` | accepted metric; empty IDs/non-finite values | 0003 / 02 |
| `cr-calibration` | monotonic curve interpolation, bounded extrapolation, drift | `cr-application::observe` | interpolation/drift; unsafe extrapolation | 0004 / 03 |
| `cr-habitat-graph` | support ledger for graph evidence (not counted) | `cr-application::record_domain_evidence` | graph append; invalid zone/score | 0005 / 04 |
| `cr-provenance` | append-only input evidence chain and verification | `cr-application::observe` | ordered chain; edited-link detection | 0004,0014 / 03,10 |
| `cr-temporal-anomaly` | rolling median/MAD detector with hysteresis | `cr-application::observe` | spike/recovery; invalid configuration/sample | 0006 / 05 |
| `cr-mesh-transport` | support ledger for route evidence (not counted) | `cr-application::record_domain_evidence` | route record; empty/invalid route | 0007 / 06 |
| `cr-neural-forecast` | support ledger for forecast evidence (not counted) | `cr-application::record_domain_evidence` | forecast record; invalid confidence/count | 0008 / 07 |
| `cr-intervention` | support ledger for action evidence (not counted) | `cr-application::record_domain_evidence` | permitted plan; invalid action/score | 0010 / 09 |
| `cr-audit` | support ledger for authorized-effect evidence (not counted) | `cr-application::record_domain_evidence` | append/chain; malformed subject/value | 0014 / 10 |
| `cr-observability` | support ledger for correlation evidence (not counted) | `cr-application::record_domain_evidence` | correlated record; missing correlation | 0016 / 11 |
| `cr-dashboard` | coherent multi-view read projection | `cr-application::dashboard` | complete projection; missing topology | 0019,0024 / 12 |
| `cr-rucelium` | real signed RuCelium ingest | `cr-upstream-compose` | signed sample verifies; invalid request fails closed | 0003 / 02 |
| `cr-worldgraph` | real worldgraph habitat snapshot | `cr-upstream-compose` | graph serialization; invalid zone | 0005 / 04 |
| `cr-midstream` | real Midstream DTW comparison | `cr-upstream-compose` | bounded comparison; empty correlation | 0006 / 05 |
| `cr-latentmesh` | real LatentMesh fragmentation/reassembly | `cr-upstream-compose` | reverse-order reassembly; invalid MTU | 0007 / 06 |
| `cr-ruvfann` | real ruv-FANN deterministic inference | `cr-upstream-compose` | forecast; wrong input shape | 0008 / 07 |
| `cr-upstream-compose` | ordered five-Ruvnet primary composition | `cr-application::intervene` | combined score; missing correlation/empty result | 0001,0017 / 01,08 |
| `cr-cedar-policy` | Cedar intervention authorization | `cr-application::intervene` | allow safe field lead; deny action/actor/malformed policy | 0011 / 09 |
| `cr-opendal-store` | OpenDAL object write/read/list | `cr-repository` | roundtrip; missing/unsafe key | 0013 / 10 |
| `cr-repository` | durable state plus full audit-chain verification on restore | `cr-application`, service startup | save/restore; audit gap, tamper, missing object | 0013,0014 / 10 |
| `cr-seed` | deterministic Toronto ravine fixture | `cr-application::new` | stable topology/recovery target | 0020 / 04 |
| `cr-application` | transactional observe/intervene/retrieve services | `cr-api` | eleven cross-crate workflow/failure tests | 0001,0010,0017 / 01,09,10 |
| `cr-middleware` | registered authn/authz/correlation/observability/error/timeout/policy/validation controls | `cr-service` and `cr-api` | route-layer order; missing/invalid bearer and role | 0012,0016 / 11 |
| `cr-api` | Axum `/api/v1` handlers and typed errors | `cr-service` router | response mapping; real handler paths | 0015 / 11 |
| `cr-service` | production composition root and health/readiness | launched binary | startup/readiness; dependency/config failure | 0017,0023 / 01,11 |

The API path reaches every counted crate through normal dependencies: `cr-service -> cr-api -> cr-application -> domain books / cr-upstream-compose / cr-cedar-policy / cr-repository`. The primary intervention reaches all seven selected upstreams. Observation admission and denial/recovery exercise complementary validation and no-effect branches. The 22-crate count excludes the seven support packages named above even though they remain on this production path.
