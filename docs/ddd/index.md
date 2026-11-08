# Domain-driven design index

| DDD | Scope | Owning crates | Primary ADRs |
|---|---|---|---|
| [01](01-context-map-and-service-boundaries.md) | contexts and service boundaries | service, application | 0001, 0017 |
| [02](02-observation-trust.md) | device provisioning, signed evidence, replay control | device-trust, rucelium, application | 0003, 0019 |
| [03](03-calibration-and-canonical-observation.md) | calibration aggregate | calibration | 0004 |
| [04](04-habitat-twin.md) | deterministic habitat state and graph projection | seed, dashboard, worldgraph | 0005, 0020 |
| [05](05-temporal-change.md) | robust detector and DTW comparison | temporal-anomaly, midstream | 0006 |
| [06](06-semantic-field-mesh.md) | constrained route fragmentation and reassembly | latentmesh, upstream-compose | 0007 |
| [07](07-neural-forecast.md) | deterministic bounded local forecast | ruvfann, upstream-compose | 0008 |
| [08](08-ecological-objective-and-evaluation.md) | named score composition and evaluation | upstream-compose, application | 0009, 0021 |
| [09](09-intervention-governance.md) | safety exclusion and Cedar authorization | application, cedar-policy | 0010, 0011 |
| [10](10-evidence-continuity.md) | OpenDAL, repository, provenance, audit restore | opendal-store, repository, provenance | 0013, 0014, 0022 |
| [11](11-product-delivery.md) | contract, middleware, API, runtime | contracts, middleware, api, service | 0002, 0012, 0015, 0016, 0023 |
| [12](12-field-operations-experience.md) | journeys, accessibility, limits | frontend, application | 0018, 0024, 0025 |

