# DDD-10: evidence continuity

## Aggregate

`Repository` is the durability aggregate over an authorized `ArtifactStore`. It owns observations, interventions, and an ordered audit. `EvidenceChain` separately links accepted observation inputs before persistence.

## Invariants

- Storage keys contain a validated namespace, kind, and object ID with no slash or traversal.
- Audit sequence starts at one and increments without gaps.
- Every audit entry names its predecessor; its hash covers sequence, event type, subject, correlation, predecessor, and payload.
- Restore verifies the full chain before returning state.

## Commands and failures

Save observation, save intervention, append audit, and restore are explicit operations. Unsafe key, missing object, deserialization failure, sequence conflict, predecessor mismatch, or edited payload produces a typed failure.

## Restart contract

The service reconstructs the repository from OpenDAL objects in a new backend process. Browser refresh and backend replacement must return the same intervention identity and audit correlation.

## Verification and reciprocal mappings

OpenDAL round trips, key failure, audit tamper tests, application restore, and browser/container restart retrieval cover the context. ADR-0013, ADR-0014, and ADR-0022 map here; owners are `cr-opendal-store`, `cr-repository`, and `cr-provenance`.
