# ADR-0007: LatentMesh transports a route over constrained frames

## Context

Field intervention routes must survive a constrained-radio boundary; a locally computed route that never enters the selected transport library would not prove offline composition.

## Decision

`cr-latentmesh` builds a route from current mesh links, serializes it, fragments it using the pinned LatentMesh BLE wire profile and `StateDelta` semantic class, feeds frames to a real reassembler in reverse order, and returns only the successfully decoded route. Missing completion or malformed input fails the workflow.

## Alternatives considered

Returning the route directly was rejected because it bypasses LatentMesh. Remote semantic retrieval was rejected because it is not the responsibility implemented by the selected revision and would overclaim the product.

## Consequences

The boundary proves fragmentation/reassembly and fallback routing, not search or radio hardware delivery. Frame count is retained as runtime evidence.

## Verification and mappings

Run `fragments_and_reassembles_route`, invalid-MTU failure, the primary journey, and the LatentMesh ablation. Reciprocal mappings: `cr-latentmesh`, `cr-upstream-compose`; DDD-06.
