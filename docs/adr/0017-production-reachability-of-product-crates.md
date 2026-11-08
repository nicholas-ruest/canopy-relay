# ADR-0017: counted crates must be production-reachable and behavior-bearing

## Context

Cargo workspaces make it easy to create named packages that compile but provide no independent product behavior. Structural counts would then reward artificial splitting.

## Decision

A counted crate needs a typed public API, useful domain responsibility, non-dev dependency path from `cr-service`, real production caller, success and failure tests, and reciprocal ADR/DDD mappings. Independent review may consolidate or exclude weak crates.

## Alternatives considered

Directory or manifest counts were rejected. LOC thresholds were rejected because verbosity is not behavior. Build/dev-only tools and adapters with no domain semantics were rejected from the count.

## Consequences

Twenty-two of twenty-nine production-reachable crates survive independent review as meaningful product crates. Seven reachable support ledgers are explicitly excluded because their repeated record shape does not establish independent domain ownership.

## Verification and mappings

Record exact locked offline Cargo metadata, trace each crate to a caller function, inspect package-owned implementation, and run mapped tests. Reciprocal mappings: every counted row in the crate map; DDD-01.

