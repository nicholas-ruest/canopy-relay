# ADR-0016: correlation and time budgets enclose every request

## Context

One browser action crosses middleware, domain, five Ruvnet boundaries, Cedar, OpenDAL, and audit. Without causal identity and deadlines, execution and denial claims cannot be proven.

## Decision

`cr-middleware` validates or replaces the correlation ID, logs the actor and registered controls, passes the ID to application calls, and returns it on the response. Application records it on interventions and audit entries. A Tower timeout bounds the whole request at 30 seconds.

## Alternatives considered

Post-hoc text matching was rejected as ambiguous. Unlimited calls were rejected. Raw payload logging was rejected because acoustic content and coordinates are sensitive. Creating correlation after auth was rejected because denial would be invisible.

## Consequences

The whole-request timeout cannot identify which inner adapter exhausted the budget, and the prototype does not claim per-upstream cancellation. Correlation still connects browser, middleware, application, storage, and audit evidence without logging sensor payloads.

## Verification and mappings

Test invalid ID replacement, successful response propagation, denial correlation, Tower timeout, and audit correlation. Reciprocal mappings: `cr-middleware`, `cr-application`, `cr-repository`; DDD-11.

