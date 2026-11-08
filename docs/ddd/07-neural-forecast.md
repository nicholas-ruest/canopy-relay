# DDD-07: neural risk forecast

## Model

`cr-ruvfann` owns a deterministic local risk forecast. Its three features are anomaly score, online-link ratio, and observation confidence. A pinned ruv-FANN network with one four-node hidden layer emits one risk value.

## Invariants

- The feature vector has exactly three finite inputs.
- Fixed weights are installed before inference so evidence is reproducible.
- Output must exist and be finite, then is bounded to `[0,1]`.
- Every `Forecast` carries zone, 24-hour horizon, uncertainty band, confidence, and immutable upstream revision.

## Authority boundary

Forecast risk contributes one score to composition. It cannot permit an intervention; Cedar and the hard safety exclusion retain authority. The model is a deterministic systems fixture, not trained ecological science.

## Failure semantics

Wrong input shape, network error, missing output, and non-finite output fail the workflow rather than substituting a safe-looking zero.

## Verification and reciprocal mappings

Real inference, wrong-shape failure, combined workflow, and the ruv-FANN ablation cover this context. ADR-0008 maps here; owners are `cr-ruvfann` and `cr-upstream-compose`.
