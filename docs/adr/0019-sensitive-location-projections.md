# ADR-0019: publish only synthetic coarse operational geometry

## Context

Exact protected-species and staff locations would be inappropriate in public evidence, and the prototype has no production identity provider for role-specific disclosure.

## Decision

The deterministic seed contains only invented Toronto-ravine zone centroids and never stores images, raw acoustic samples, people, or protected-species coordinates. The API exposes one coarse operational projection to authenticated prototype users. The README and UI state explicitly that these coordinates are synthetic and that role-specific fine-grained projection is not implemented.

## Alternatives considered

Copying real habitat data, pretending frontend redaction is authorization, and claiming unimplemented role-aware projection were rejected.

## Consequences

The product proves workflow and evidence boundaries but not a production sensitive-location policy. Any real deployment needs classified geometry and backend projection rules.

## Verification and mappings

Inspect `cr-seed`, API DTOs, public evidence, and limitation text for absence of real sensitive records. Reciprocal mappings: `cr-seed`, `cr-dashboard`, `cr-contracts`; DDD-02 and DDD-12.
