# Prototype acceptance

## `[journey:primary]` — trusted evidence and governed relay

The operator loads the real dashboard, submits a sensor reading through `POST /api/v1/observations`, and receives a rendered observation ID. Middleware authenticates and correlates the request; `CanopyRelay::observe` validates the DTO, admits a one-use device receipt, applies bounded calibration, runs robust anomaly scoring, verifies the provenance chain, persists through OpenDAL, and appends audit.

The operator then submits `deploy-mesh-relay` through `POST /api/v1/interventions`. Middleware requires `field-lead`; the application validates and asks Cedar before executing RuCelium, WorldGraph, Midstream, LatentMesh, and ruv-FANN. It persists one intervention and one linked audit entry, and the React planner renders route, predicted effect, policy decision, correlation, and evidence hash.

Acceptance: all seven selected upstreams execute across the workflow; the browser renders backend-issued identities; correlation joins UI, middleware, application, upstream, and storage logs.

## `[journey:denial-recovery]`

The operator submits `controlled-burn`. The application rejects this explicitly unsupported irreversible action before Cedar, selected upstreams, intervention storage, or audit. The test compares intervention and audit counts before and after. The UI renders denial rather than success. The operator changes the request to `deploy-mesh-relay`; middleware, Cedar, all selected upstreams, and OpenDAL execute, and one result appears.

Acceptance: forbidden durable effects remain absent; the corrected action succeeds through the same real stack; both outcomes retain correlation evidence.

## `[journey:durable-retrieval]`

The browser records an approved intervention identity and audit correlation, refreshes, and retrieves it. The test stops the Rust process and starts a new `cr-service` over the same OpenDAL filesystem. `Repository::restore` lists objects, rebuilds state, and recomputes the full audit chain before readiness. The browser retrieves the same intervention and audit after the new process is ready.

Acceptance: process replacement is real, no in-memory fallback supplies the record, and identity plus correlation remain stable.

## Evidence binding

All three titles execute on desktop and mobile against built React and the real Rust backend. External exact-source evidence includes Playwright JSON and traces, desktop/mobile screenshots, browser and backend logs, correlated upstream trace, restart receipt, and hashes in schema-v2 `manifest.json`.
