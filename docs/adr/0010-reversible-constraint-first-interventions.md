# ADR-0010: reject irreversible intervention classes before composition

## Context

The synthetic product can demonstrate reversible field relay and calibration work, but it cannot establish safety for burning or species release.

## Decision

`CanopyRelay::intervene` validates the typed request and immediately denies `controlled-burn` and `release-species` before Cedar, upstream execution, persistence, or audit effects. Remaining actions must pass Cedar and the composed workflow before one intervention and one audit entry are committed.

## Alternatives considered

Letting a model invent actions and compensating after a denied write were rejected. A blanket no-op intervention mode was rejected because the primary journey must produce a real durable outcome.

## Consequences

This is a bounded prototype catalog, not ecological authorization guidance. Adding a high-impact action requires explicit code and policy review.

## Verification and mappings

The denial/recovery browser journey and `restricted_action_is_denied_before_upstream_or_persistence_effects` compare durable effects before and after denial. Reciprocal mappings: `cr-application`, `cr-contracts`; DDD-09.
