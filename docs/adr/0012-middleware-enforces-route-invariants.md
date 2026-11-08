# ADR-0012: route controls and effect policy are separate executable layers

## Context

Every product route needs consistent correlation and authentication, while mutations additionally need role and content checks. Ecological policy must still run at the application effect boundary.

## Decision

Axum registers CORS, tracing, a 30-second timeout, and `product_controls`. That middleware bypasses only health routes, normalizes correlation, authenticates a bearer token, requires `field-lead` for intervention POST, requires JSON for mutations, and returns correlated problem responses. Handlers repeat typed authentication to construct request context; Cedar runs inside `CanopyRelay::intervene`.

## Alternatives considered

Ad-hoc uncorrelated handler errors and policy only in middleware were rejected. A separate BFF was rejected because it adds no product boundary.

## Consequences

Prototype bearer parsing is deliberately local and not an identity-provider claim. The registration and behavioral tests make the actual order inspectable.

## Verification and mappings

Run middleware missing-token, wrong-role, content-type, correlation, timeout, and handler error tests; inspect `cr-service::run`. Reciprocal mappings: `cr-middleware`, `cr-api`, `cr-service`; DDD-11.
