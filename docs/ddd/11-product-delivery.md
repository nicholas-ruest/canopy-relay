# DDD-11: product delivery

## Boundary

Product Delivery owns the `/api/v1` HTTP contract, checked TypeScript client, Axum service topology, middleware controls, health/readiness endpoints, and container/local scripts. It translates but does not authorize ecological effects.

## Invariants

- Non-health routes require a bearer token and a valid/generated correlation ID.
- Intervention POST requires `field-lead`; all POST routes require JSON.
- Tower tracing and 30-second timeout enclose the product middleware and handler.
- API envelopes are versioned; the client paths and DTOs must match the checked OpenAPI schema.

## Runtime model

`cr-service` restores storage and compiles Cedar before binding. `/healthz` proves process liveness; `/readyz` is available only after successful construction. The same binary supplies a `--healthcheck` probe for containers.

## Failure semantics

Middleware returns correlated problems. Handler errors map typed domain variants to stable statuses. Startup construction errors terminate the process rather than accepting traffic.

## Verification and reciprocal mappings

Middleware tests, API drift check, frontend typecheck, readiness tests, container launch, and browser journeys cover the boundary. ADR-0002, ADR-0012, ADR-0015, ADR-0016, and ADR-0023 map here.
