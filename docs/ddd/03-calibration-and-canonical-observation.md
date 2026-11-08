# DDD-03: calibration and canonical observation

## Aggregate

An `Observation` is produced from one validated `ObservationInput`, one named `CalibrationCurve`, one device receipt, and one evidence-chain link. The raw input remains embedded while `calibrated_value`, confidence, and provenance hash record the derived state.

## Invariants

- Calibration points are finite and strictly increasing by raw value.
- A reading outside the curve plus five-percent extrapolation is rejected.
- Calibration completes before the repository sees the observation.
- The evidence-chain hash covers the serialized input and survives into the observation.

## Current scope

The synthetic field UI submits ratio signals and the production path uses a versioned identity curve over `[0,1]`. The crate also implements drift comparison between two curve revisions. Unit catalogs and time-ranged profile lookup are explicitly out of scope.

## Failure semantics

Invalid location or request shape fails in `cr-contracts`; malformed calibration or unsafe extrapolation fails in `cr-calibration`; no partial observation or audit is written.

## Verification and reciprocal mappings

Curve interpolation/drift/failure tests and `trusted_observation_is_persisted_and_hash_linked_into_audit` cover the context. ADR-0004 maps here; owners are `cr-calibration`, `cr-contracts`, and `cr-application`.
