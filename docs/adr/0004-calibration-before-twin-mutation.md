# ADR-0004: bounded calibration precedes observation persistence

## Context

Non-finite and out-of-envelope readings must not become durable observations. Calibration must occur in the backend rather than only in presentation code.

## Decision

`CanopyRelay::observe` creates a versioned monotonic `CalibrationCurve`, applies it before constructing the observation, and rejects values beyond five-percent extrapolation. The current field fixture uses an identity ratio curve over `[0,1]`; the curve revision is still carried in the observation and calibration ledger.

## Alternatives considered

Frontend normalization was rejected because direct API callers would bypass it. Unbounded extrapolation was rejected because it turns an invalid reading into apparently precise evidence.

## Consequences

The implemented fixture covers ratio sensors, not a general unit-conversion catalog. Adding new sensor kinds requires explicit curves and contract tests.

## Verification and mappings

Run calibration interpolation, drift, and unsafe extrapolation tests plus trusted observation acceptance. Reciprocal mappings: `cr-calibration`, `cr-application`, `cr-types`; DDD-03.
