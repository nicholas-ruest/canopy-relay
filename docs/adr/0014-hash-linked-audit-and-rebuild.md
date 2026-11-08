# ADR-0014: verify the hash-linked audit before restored state is served

## Context

Sorted durable audit objects could be deleted, reordered, or edited while still deserializing successfully.

## Decision

Each application audit hash covers sequence, event type, subject, correlation, predecessor hash, and payload bytes. `Repository::restore` sorts sequence-prefixed objects, then recomputes every hash and checks contiguous sequence and predecessor linkage before returning the repository. Any mismatch fails service startup; dashboard projection never sees the corrupted chain.

## Alternatives considered

Trusting filenames, silently truncating to a valid prefix, and using the dashboard as source of truth were rejected.

## Consequences

The prototype has no snapshot compaction and restore cost grows with audit length. The benefit is deterministic tamper detection and restart continuity.

## Verification and mappings

Run ordered/tampered chain tests and the restart-persistent browser journey. Reciprocal mappings: `cr-repository`, `cr-application`, `cr-dashboard`; DDD-10.
