# ADR-0011: Cedar authorization runs before selected upstreams and writes

## Context

Route roles alone cannot protect internal callers or express the action/resource decision at the effect boundary.

## Decision

After request validation and the hard safety exclusion, `cr-application` asks the pinned Cedar engine to authorize principal, action, and zone. It records the audit count before evaluation and refuses to continue if policy evaluation changed state. A denial produces no intervention and no audit event; an allowed workflow commits only after all five Ruvnet boundaries succeed.

## Alternatives considered

UI-only permission checks, policy after persistence, and permissive behavior on engine error were rejected.

## Consequences

Denied attempts are visible in correlated middleware logs rather than the durable ecological audit. A separate security-audit sink would be required in production.

## Verification and mappings

Run wrong-actor, restricted-action, malformed-policy, and browser denial/recovery tests. Reciprocal mappings: `cr-cedar-policy`, `cr-application`, `cr-middleware`; DDD-09.
