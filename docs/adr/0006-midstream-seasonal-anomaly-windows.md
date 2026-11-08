# ADR-0006: Midstream compares baseline and current habitat sequences

## Context

An intervention score needs temporal change evidence rather than a single threshold.

## Decision

`cr-midstream` derives a deterministic baseline at ninety percent of the current anomaly series, constructs real Midstream `Sequence` values, runs DTW through `TemporalComparator`, and converts distance to a bounded similarity score. Empty correlation or zone fails before comparison.

## Alternatives considered

A constant score and local ad-hoc distance function were rejected because they would not execute the selected upstream. Missing-value imputation was not introduced because the fixture has an explicit series.

## Consequences

The synthetic comparison is reproducible but is not a learned seasonal baseline. Its evidence records algorithm, distance, and point count so that limitation is visible.

## Verification and mappings

Run `compares_temporal_series`, invalid-request tests, benchmark, and the Midstream ablation. Reciprocal mappings: `cr-midstream`, `cr-upstream-compose`; DDD-05.
