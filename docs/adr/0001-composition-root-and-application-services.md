# ADR-0001: one runtime composition root and one application boundary

## Context

Observation ingest, intervention planning, dashboard projection, and audit retrieval must use the same policy, upstream clients, repository, and deterministic habitat seed. Constructing these dependencies in handlers would make denial and restart behavior diverge.

## Decision

`cr-service::run` constructs the OpenDAL store, restoring repository, Cedar authorizer, five Ruvnet clients, and one `CanopyRelay`. `cr-api` only maps HTTP DTOs to the `observe`, `intervene`, dashboard, and retrieval methods. `CanopyRelay` owns effect ordering and is the sole production caller of composed domain behavior.

## Alternatives considered

Direct handler orchestration was rejected because it duplicates effect ordering. A service locator was rejected because dependencies disappear from the type graph. A monolithic binary was rejected because domain behavior would be inseparable from transport.

## Consequences

The application object is intentionally stateful for replay, anomaly, and evidence-chain guards. Integration tests can call the same boundary as HTTP without creating a second topology.

## Verification and mappings

Inspect `cr-service -> cr-api -> cr-application` in locked metadata and source; run `cr-application/tests/full_workflows.rs`. Reciprocal mappings: `cr-service`, `cr-api`, `cr-application`; DDD-01.
