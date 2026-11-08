# DDD-01: context map and service boundaries

## Model

Canopy Relay has six bounded contexts: Observation Trust, Habitat State, Temporal Intelligence, Field Transport, Intervention Governance, and Evidence Continuity. Product Delivery translates HTTP and browser concerns but does not own ecological rules.

`cr-application` is the application boundary. Observation Trust publishes an accepted `Observation`; the intervention workflow reads the deterministic `SeedWorld`, composes five upstream results, asks Governance for authorization, and asks Evidence Continuity to persist the result and linked audit.

## Invariants

- HTTP handlers never construct upstreams or write storage directly.
- A denied intervention cannot enter Field Transport, persistence, or audit.
- A dashboard intervention must have a matching audit subject.
- Every selected upstream result shares the request correlation ID at its adapter boundary.

## Language and ownership

Observation, habitat zone, anomaly, mesh link, forecast, intervention, and audit entry are the shared kernel in `cr-types`. `cr-contracts` owns transport envelopes; it does not redefine domain entities. `cr-service` owns composition, not business rules.

## Verification and reciprocal mappings

Source and locked metadata prove `cr-service -> cr-api -> cr-application -> domain/adapters`. Full-workflow tests prove success, denial with zero effects, and restore. ADR-0001 and ADR-0017 map here.
