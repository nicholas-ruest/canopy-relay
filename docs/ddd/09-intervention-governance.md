# DDD-09: intervention governance

## Aggregate

An `Intervention` binds one typed request, authorized decision, selected route, predicted effect, correlation ID, evidence hash, state, and creation time. The application creates it only after safety, Cedar, and all selected upstream checks succeed.

## Invariants

- `controlled-burn` and `release-species` are denied before any downstream effect.
- The caller must hold the middleware `field-lead` role and pass Cedar as `field-lead` at the effect boundary.
- Policy evaluation cannot change durable audit count.
- A successful commit appends exactly one intervention and one audit event.

## Commands and states

`intervene` is the command. The current prototype produces `deployed` or a typed failure; it does not simulate a long-running lifecycle. `deploy-mesh-relay` and `schedule-calibration` are demonstrated reversible actions.

## Failure semantics

Validation, hard denial, Cedar denial, upstream error, invalid score, storage error, or audit-count mismatch prevents a successful response.

## Verification and reciprocal mappings

Wrong-actor, restricted-action, recovery, malformed-policy, primary, and browser denial tests cover this context. ADR-0010 and ADR-0011 map here; owners are `cr-cedar-policy` and `cr-application`.
