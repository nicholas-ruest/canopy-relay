# Integration matrix

Each row is one canonical upstream with an immutable revision, license, distinct executed responsibility, production call site, success/failure proof, and ablation.

| Canonical upstream | Immutable revision | License | Executed responsibility | Production owner/call site | Success and failure proof | Ablation |
|---|---|---|---|---|---|---|
| `ruvnet/RuCelium` | `813da13eb9a46b6acf78fe7ecc8c220ecf83b386` | MIT | sign, register, ingest, and verify a real environmental sample | `cr-rucelium::Client`, called in `cr-upstream-compose::execute` | verified provenance / empty correlation rejected; tampered probe rejected | `without_rucelium` |
| `ruvnet/worldgraph` | `9b1c79c836cdacfb7b44f058c593157bac4c1dab` | MIT OR Apache-2.0 | construct and serialize a bounded zone node | `cr-worldgraph::Client`, called in composition | node/snapshot evidence / empty zone rejected | `without_worldgraph` |
| `ruvnet/midstream` | `62c37c9e8d282a4249de41aaf63649230338f738` | MIT OR Apache-2.0 | run DTW over deterministic baseline/current sequences | `cr-midstream::Client`, called in composition | finite distance / empty correlation rejected | `without_midstream` |
| `ruvnet/LatentMesh` | `df9270799a1391925c1840a1845a6d169aaa2d93` | MIT OR Apache-2.0 | fragment and reassemble a constrained-radio route | `cr-latentmesh::Client`, called in composition | decoded route / invalid MTU and incomplete message fail | `without_latentmesh` |
| `ruvnet/ruv-FANN` | `f4ca36c2323d08eac822bac43b62040198721cbf` | MIT OR Apache-2.0 | execute a deterministic three-feature neural risk forecast | `cr-ruvfann::Client`, called in composition | bounded forecast / wrong input shape fails | `without_ruvfann` |
| `cedar-policy/cedar` | `5af7b8cd78e4e0da3ae23f1715023fff8e62941d` | Apache-2.0 | authorize actor/action/zone before selected upstreams or writes | `cr-cedar-policy::CedarAuthorizer`, called in `CanopyRelay::intervene` | field-lead allow / observer, restricted action, malformed policy fail | `without_cedar` |
| `apache/opendal` | `532050ac3c099dd8591cde652a8ed1aaf03ba418` | Apache-2.0 | typed namespaced write/read/list and restart restoration | `cr-opendal-store::ArtifactStore`, called by `cr-repository` | memory/filesystem round trip and reopen / unsafe key or missing object fail | `without_opendal` |

The primary intervention executes all seven boundaries. Observation ingest additionally exercises the product-owned trust, calibration, anomaly, provenance, and persistence behaviors. Probe archives and final runtime receipts bind these exact identities and revisions.
