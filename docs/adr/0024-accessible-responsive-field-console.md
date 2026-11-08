# ADR-0024: a multi-view responsive field console over the real API

## Context

A generic form or static dashboard would not let operators inspect habitat, trust, anomaly, intervention, mesh, and audit state across laptop and phone layouts.

## Decision

React provides six labeled views with semantic navigation, keyboard-operable controls, real API loading/error/success feedback, observation analysis, governed intervention planning, dependency status, and durable audit receipts. CSS supplies narrow-screen layout and reduced-motion handling; text and status labels accompany color.

## Alternatives considered

A map-only canvas, mock API, one-form demo, and optimistic success before backend response were rejected.

## Consequences

The frontend is an operations console, not a GIS editor. Accessibility is covered by named-control and keyboard assertions but is not claimed as a formal external audit.

## Verification and mappings

Run frontend unit/accessibility checks and desktop/mobile Playwright journeys with actual screenshots and traces. Reciprocal mappings: frontend, `cr-contracts`, `cr-dashboard`; DDD-12.
