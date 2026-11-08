# ADR-0005: WorldGraph serializes the intervention habitat cell

## Context

The composed workflow needs machine-readable spatial evidence instead of a decorative dependency declaration.

## Decision

`cr-worldgraph` constructs a pinned `WorldGraph`, inserts the requested habitat zone as a bounded room node with a stable derived ID, serializes the graph, and returns node count and snapshot size as runtime evidence. The adapter rejects missing correlation or zone before invoking the upstream.

## Alternatives considered

A hand-written JSON graph and an unused manifest dependency were rejected because neither executes WorldGraph. Building a full ecological GIS was rejected as unsupported by the synthetic prototype.

## Consequences

This boundary proves a real graph construction and serialization responsibility, not multi-hop ecological propagation. Future topology expansion must add explicit link semantics and tests.

## Verification and mappings

Run `constructs_real_graph` and the empty-zone failure test; inspect the correlated composition evidence. Reciprocal mappings: `cr-worldgraph`, `cr-upstream-compose`; DDD-04.
