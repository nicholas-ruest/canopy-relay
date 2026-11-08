# ADR-0018: acceptance is three real browser journeys

## Context

Unit and API tests cannot prove that a field operator can complete work, that denial prevents effects, or that persistence survives process replacement.

## Decision

Playwright drives the built React client and real Rust service for primary habitat decision, denial/recovery, and durable retrieval after refresh plus backend restart. Required titles, desktop/mobile screenshots, traces, correlations, and raw logs are retained.

## Alternatives considered

A static preview, mocked first-party route, API-only suite, and one generic form were rejected because they bypass product composition. Restarting only the browser was rejected as durability evidence.

## Consequences

The suite is slower and more operationally demanding, but it validates navigation, middleware, domain, upstream, storage, and rendering as one product.

## Verification and mappings

Assert all journey titles execute, denial effect counters remain unchanged, the corrected action succeeds, restart is a new process, and stable receipt fields match. Reciprocal mappings: frontend, `cr-api`, `cr-application`, `cr-opendal-store`; DDD-12.

