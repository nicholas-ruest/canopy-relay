# ADR-0009: fuse five named upstream scores without hiding ingredients

## Context

The intervention path needs one bounded prediction while preserving which selected upstreams contributed.

## Decision

`CompositionResult` stores separate RuCelium quality, WorldGraph graph, Midstream temporal, LatentMesh transport, and ruv-FANN confidence scores plus each upstream evidence object. `combined_score` rejects non-finite values and computes their unweighted arithmetic mean. Cedar authorization remains separate and cannot be overridden by the score.

## Alternatives considered

An opaque learned ranker and frontend-only fusion were rejected. Unreviewed weights were rejected; the equal mean is intentionally simple and its limitation is explicit.

## Consequences

Different score semantics are normalized but not scientifically calibrated. Named ablations expose the contribution and capability loss of every ingredient.

## Verification and mappings

Run composition success, non-finite rejection tests, benchmark rows, and seven ablations. Reciprocal mappings: `cr-upstream-compose`, `cr-application`; DDD-08.
