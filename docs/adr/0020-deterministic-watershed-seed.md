# ADR-0020: one deterministic synthetic ravine fixture

## Context

Browser evidence, upstream composition, benchmark rows, and restart retrieval need repeatable inputs without importing real ecological data.

## Decision

`cr-seed::toronto_ravine` returns three invented zones, two sensors, one open mesh-loss anomaly, and two links at a fixed timestamp. Application, tests, benchmark, and ablations use this same typed fixture. Reset scripts remove only the configured Canopy Relay data directory after validating the project path.

## Alternatives considered

Production telemetry and unrecorded random generation were rejected. Separate unrelated fixtures for each suite were rejected because results would not be comparable.

## Consequences

The seed demonstrates system behavior, not ecological generalization. Changes to it alter benchmark and browser expectations and require a new exact-source evidence run.

## Verification and mappings

Run seed connectivity tests, reset-path safety tests, benchmark, and all three browser journeys. Reciprocal mappings: `cr-seed`, `cr-application`, delivery scripts; DDD-04.
