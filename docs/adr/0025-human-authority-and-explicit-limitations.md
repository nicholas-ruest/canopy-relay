# ADR-0025: retain human authority and publish limitations

## Context

Sensor bias, stale topology, outdated field notes, and out-of-domain neural inference can all produce plausible but wrong recommendations. Ecological harm can be irreversible.

## Decision

Canopy Relay rejects two irreversible action classes in code and requires a `field-lead` role plus Cedar authorization for every permitted intervention. Upstream scores and the neural forecast are evidence, not authorization. README and product views expose synthetic-fixture scope, privacy limits, storage limits, and absence of causal or species-identification claims.

## Alternatives considered

Autonomous irreversible action and hidden confidence thresholds were rejected. Treating gate success as scientific validation was rejected. Generic legal disclaimers without product-specific failure modes were rejected.

## Consequences

Automation is intentionally bounded and some workflows stop for abstention or review. Trust improves because limitations remain visible at decision time.

## Verification and mappings

Test wrong-role and restricted-action denial with zero effects, permitted recovery, and rendered limitation text. Reciprocal mappings: `cr-application`, `cr-cedar-policy`, frontend/README; DDD-12.

