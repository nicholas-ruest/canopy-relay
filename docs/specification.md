# Product specification

## Thesis and users

Canopy Relay is a camera-free operations console for synthetic urban-ravine monitoring. It lets a field technician submit trusted sensor evidence, lets a habitat steward evaluate and authorize a reversible relay response, and lets a reviewer retrieve the resulting evidence after browser and backend restart.

The deterministic fixture contains three invented habitat zones, two sensors, one mesh-loss anomaly, and two connectivity links. It is safe to publish and is not represented as field science.

## Functional requirements

1. Accept observations only from a provisioned device with a one-use signed-ingest receipt.
2. Validate location and signal shape, apply bounded backend calibration, run robust per-signal anomaly scoring, and append provenance before persistence.
3. Compose a real RuCelium signed sample, WorldGraph node, Midstream DTW comparison, LatentMesh fragmentation/reassembly, and ruv-FANN forecast for the intervention workflow.
4. Preserve each upstream's evidence and compute a finite five-part mean without granting it authorization authority.
5. Deny explicitly irreversible actions before downstream effects and evaluate all remaining interventions with Cedar before upstreams or storage.
6. Persist observations, interventions, and linked audit entries through OpenDAL and verify the audit chain on restore.
7. Expose `/api/v1` through executable authentication, role, validation, correlation, timeout, error, and observability controls plus a checked TypeScript client.
8. Render six responsive React views using real backend state, with loading, error, success, denial, recovery, and audit receipt feedback.
9. Execute primary, denial/recovery, and browser-refresh plus backend-restart retrieval journeys.

## Quality and success requirements

- Denial leaves intervention and audit counts unchanged; corrected recovery appends exactly one of each.
- Replayed device receipts and unsafe calibration values fail before durable effects.
- The default production binary reaches all counted crates through normal non-dev edges.
- The exact-source verifier records all ten suites, Cargo metadata, browser traces/screenshots, container restart, benchmark, and seven ingredient ablations.
- Rust comprises at least 50% of recognized tracked source bytes without hygiene failures.

## Limitations and non-goals

The fixed neural weights, coarse WorldGraph node, DTW baseline, and equal-weight score are integration mechanisms, not validated ecological models. Canopy Relay does not perform camera analytics, species identification, causal inference, real radio delivery, autonomous irreversible action, or role-aware sensitive-location release. The prototype bearer format and filesystem OpenDAL backend are not production identity or multi-region storage.
