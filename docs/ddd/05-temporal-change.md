# DDD-05: temporal change

## Model

Temporal evidence has two layers. Observation ingest owns a per-zone-and-metric `RobustDetector` using a rolling median, median absolute deviation, and enter/exit hysteresis. Intervention composition owns a real Midstream DTW comparison between deterministic baseline and current anomaly sequences.

## Invariants

- Detector configuration needs at least five samples and an enter threshold above exit.
- Non-finite readings fail before state update.
- The detector key combines habitat zone and metric so unrelated signals do not share history.
- Midstream requires correlation and zone and produces a finite bounded similarity score.

## Decisions and failures

The rolling detector modifies observation confidence but does not independently authorize intervention. Midstream distance is one named ingredient in the combined score. Invalid detector thresholds, invalid samples, or upstream errors stop the calling workflow.

## Verification and reciprocal mappings

Spike/recovery and invalid-sample tests cover the detector; real DTW and invalid-request tests cover Midstream. ADR-0006 maps here; owners are `cr-temporal-anomaly`, `cr-midstream`, and `cr-upstream-compose`.
