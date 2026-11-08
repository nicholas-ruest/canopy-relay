# ADR-0002: typed failures and bounded service configuration

## Context

Operators and the browser must distinguish unauthenticated, denied, invalid, conflicting, timed-out, storage, integrity, and dependency failures. The service also needs a reproducible address and storage root.

## Decision

All crates return `cr_errors::RelayError`; `cr-api` maps its variants to stable HTTP statuses and problem bodies. `cr-service` reads only `CANOPY_RELAY_ADDR` and `CANOPY_RELAY_DATA`, parses the address before binding, constructs the OpenDAL filesystem root, compiles the embedded Cedar policy, and refuses startup if restoration or construction fails.

## Alternatives considered

Free-form string errors were rejected because clients cannot classify them. Permissive policy fallback and in-handler environment reads were rejected because behavior would vary by route.

## Consequences

The configuration surface is deliberately small. Storage backend and policy changes require code/config review rather than silent runtime substitution.

## Verification and mappings

Run error-mapping tests, malformed-policy tests, unsafe storage-key tests, and readiness tests. Reciprocal mappings: `cr-errors`, `cr-service`, `cr-api`; DDD-11.
