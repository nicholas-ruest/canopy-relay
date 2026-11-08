# DDD-04: habitat state and graph projection

## Aggregate

`SeedWorld` is the deterministic habitat-state aggregate: three zones, two sensors, one anomaly, and two mesh links. `DashboardBuilder` validates that sensor and anomaly zone references exist and that every projected intervention has durable audit evidence.

## WorldGraph boundary

For an intervention, `cr-worldgraph` maps the target zone to one real upstream room node with stable derived identity and bounded ENU geometry, then serializes the graph. Its evidence reports the node count, snapshot size, and zone.

## Invariants

- Habitat zone IDs are unique and non-empty in a dashboard.
- Sensors and anomalies cannot reference unknown zones.
- Persisted interventions without audit evidence cannot render.
- Graph invocation requires correlation and zone.

## Scope

The product is a camera-free operational twin over synthetic coarse geometry. It does not claim multi-hop GIS propagation or real protected-species locations.

## Verification and reciprocal mappings

Seed connectivity, dashboard integrity, WorldGraph success/failure, and primary browser rendering cover the context. ADR-0005 and ADR-0020 map here; owners are `cr-seed`, `cr-dashboard`, and `cr-worldgraph`.
