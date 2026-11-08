# ADR-0015: OpenAPI v1 and a checked TypeScript client share one contract

## Context

Hand-maintained frontend interfaces drift from Rust responses, particularly for denial, abstention, and dependency-error variants.

## Decision

`cr-contracts` owns transport DTOs and schema generation separate from domain entities. Routes live under `/api/v1`; the TypeScript client is generated or mechanically checked, and CI fails when committed schema/client and Rust types disagree.

## Alternatives considered

Unversioned JSON was rejected because compatibility cannot be reasoned about. Duplicate handwritten types were rejected. Serializing domain entities directly was rejected because internal evolution would become public compatibility.

## Consequences

Explicit mapping code and version review add work. Browser, API, and documentation gain a common discriminated language for states and failures.

## Verification and mappings

Run schema generation twice for determinism, the drift check, TypeScript compilation, representative success/error roundtrips, and a breaking-change fixture. Reciprocal mappings: `cr-contracts`, `cr-api`, frontend client; DDD-11.

