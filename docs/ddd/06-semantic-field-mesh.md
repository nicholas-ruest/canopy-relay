# DDD-06: constrained field transport

## Aggregate

A field route is an ordered list of habitat or gateway identifiers derived from current `MeshLink` state. It becomes a `StateDelta` message transported through the selected LatentMesh wire boundary.

## Invariants

- The route begins at the requested zone.
- Only links explicitly marked online extend the route; otherwise the deterministic field-gateway fallback is used.
- Serialized route bytes must fragment at the BLE MTU and fully reassemble before they are trusted.
- An empty decoded route cannot advance intervention composition.

## Commands and failures

`fragment_message` produces frames with stream, sequence, semantic class, priority, and state tag. `Reassembler` accepts them in reverse order to prove ordering tolerance. Invalid MTU, malformed frame, incomplete message, or decode failure is an upstream/integrity failure.

## Scope

This context models constrained message transport, not semantic search, live radio transmission, or network availability.

## Verification and reciprocal mappings

Real fragmentation/reassembly, invalid-MTU, primary journey, and LatentMesh ablation cover the context. ADR-0007 maps here; owners are `cr-latentmesh` and `cr-upstream-compose`.
