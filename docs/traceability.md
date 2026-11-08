# Requirement traceability

| Requirement | Decision/domain source | Production owner | Executed evidence |
|---|---|---|---|
| signed ingest and receipt replay control | ADR-0003, DDD-02 | `cr-rucelium`, `cr-device-trust`, `cr-application` | real signed sample; malformed/replayed receipt failures |
| bounded backend calibration | ADR-0004, DDD-03 | `cr-calibration`, `cr-application` | interpolation/drift tests and accepted observation |
| habitat graph evidence | ADR-0005, DDD-04 | `cr-worldgraph`, `cr-dashboard` | real node serialization and invalid zone failure |
| robust and DTW temporal change | ADR-0006, DDD-05 | `cr-temporal-anomaly`, `cr-midstream` | spike/recovery plus real DTW and failure |
| constrained field transport | ADR-0007, DDD-06 | `cr-latentmesh` | fragmentation/reassembly and invalid MTU |
| bounded local forecast | ADR-0008, DDD-07 | `cr-ruvfann` | real inference and wrong-shape failure |
| named five-score composition | ADR-0009, DDD-08 | `cr-upstream-compose` | finite mean, evidence vector, ingredient ablations |
| denial before effects | ADR-0010/0011, DDD-09 | `cr-cedar-policy`, `cr-application` | zero-effect denial and permitted recovery |
| executable request controls | ADR-0012/0016, DDD-11 | `cr-middleware`, `cr-service` | authn/authz/content/correlation/timeout tests |
| durable verified restoration | ADR-0013/0014, DDD-10 | `cr-opendal-store`, `cr-repository` | round trip, tamper rejection, backend restart retrieval |
| typed API/client | ADR-0015, DDD-11 | `cr-contracts`, `cr-api`, frontend client | schema/client drift test and TypeScript build |
| meaningful production graph | ADR-0017, DDD-01 | `cr-service`, `cr-application` | locked offline Cargo metadata and semantic review |
| three browser journeys | ADR-0018, DDD-12 | frontend and complete backend | Playwright JSON/traces/screenshots/correlation logs |
| synthetic privacy boundary | ADR-0019, DDD-02/12 | `cr-seed`, `cr-dashboard` | public tree inspection and rendered limitations |
| deterministic fixture | ADR-0020, DDD-04 | `cr-seed` | seed tests, benchmark, reset |
| benchmark and ablations | ADR-0021, DDD-08 | `cr-application` example | combined/baselines/seven removals |
| exact-source evidence | ADR-0022, DDD-10 | evidence scripts | schema-v2 manifest, receipts, hashes, gates |
| container topology | ADR-0023, DDD-11 | service/frontend containers | build, readiness, primary, denial, restart receipt |
| responsive field product | ADR-0024, DDD-12 | React frontend | unit/a11y plus desktop/mobile journeys |
| human authority and limitations | ADR-0025, DDD-09/12 | application, Cedar, README | restricted action denial and visible caveats |

The [crate call-path map](crate-to-backend-call-path.md) provides the reciprocal per-crate mapping. Evidence rows describe executed checks; final hashes live only in the external exact-source artifact.
