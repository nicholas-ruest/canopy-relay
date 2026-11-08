# ADR-0008: ruv-FANN produces a deterministic bounded risk forecast

## Context

The composed intervention needs local inference with an inspectable input shape and reproducible weights.

## Decision

`cr-ruvfann` creates a three-input, four-hidden, one-output network using the pinned crate, installs deterministic weights, and runs anomaly score, online-link ratio, and observation confidence through it. Output must be finite, is clamped to `[0,1]`, and becomes a 24-hour forecast with an explicit uncertainty band and model revision.

## Alternatives considered

A cloud model and a hard-coded scalar were rejected. Dynamic model training was rejected because the fixture has no scientifically valid training corpus.

## Consequences

The result is an executable deterministic inference fixture, not a validated ecological predictor. Wrong input shape and empty output fail closed.

## Verification and mappings

Run the real forecast and wrong-shape tests, inspect forecast evidence, and execute the ruv-FANN ablation. Reciprocal mappings: `cr-ruvfann`, `cr-upstream-compose`; DDD-07.
