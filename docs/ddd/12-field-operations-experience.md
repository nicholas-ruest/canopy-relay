# DDD-12: field operations experience

## Product model

The React console has six domain views: Habitat Map, Sensor Trust, Anomaly Workbench, Intervention Planner, Offline Mesh, and Audit Vault. One authenticated session operates over real backend state.

## Invariants

- Navigation, forms, tables, and buttons have accessible names and keyboard operation.
- Color is supplemented by text, labels, percentages, or status icons.
- Loading, error, and success feedback cannot be mistaken for one another.
- A rendered observation or intervention uses the backend receipt; the client never fabricates success.

## Journeys

Primary analyzes a signed observation and deploys a governed relay. Denial/recovery proves a restricted action leaves effect counts unchanged before a corrected action succeeds. Durable retrieval refreshes the browser, replaces the backend process, and retrieves the same saved intervention/audit.

## Responsive behavior and scope

The narrow layout preserves navigation and controls, and reduced-motion CSS disables nonessential animation. The product is an operational console over synthetic geometry, not a full GIS editor or offline service worker.

## Verification and reciprocal mappings

Frontend unit tests and desktop/mobile Playwright journeys with screenshots, JSON, trace, and correlated backend logs cover this context. ADR-0018, ADR-0019, ADR-0024, and ADR-0025 map here.
