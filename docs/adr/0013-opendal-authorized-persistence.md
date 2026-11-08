# ADR-0013: one namespaced OpenDAL persistence boundary

## Context

Restart retrieval needs durable product state without scattering filesystem calls through domain crates.

## Decision

`cr-opendal-store::ArtifactStore` owns Memory and filesystem operators, validates namespace/kind/id segments against empty, slash, and traversal values, serializes canonical Rust values to JSON, and exposes typed put/get/list operations. `cr-repository` is the only application-facing owner and stores observations, interventions, and sequence-prefixed audit objects.

## Alternatives considered

In-memory-only acceptance and direct filesystem access were rejected. A bespoke database was rejected because OpenDAL already supplies the authorized storage abstraction.

## Consequences

The local filesystem backend does not claim distributed transactions. Application ordering and audit verification provide the prototype's consistency boundary.

## Verification and mappings

Run OpenDAL memory/filesystem round trips, unsafe-key failures, restart restoration, and container volume restart. Reciprocal mappings: `cr-opendal-store`, `cr-repository`, `cr-application`; DDD-10.
