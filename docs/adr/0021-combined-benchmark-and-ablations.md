# ADR-0021: execute one combined baseline and explicit capability-loss ablations

## Context

Seven upstreams are justified only if each has a visible runtime responsibility and its removal has a named product consequence.

## Decision

The `cr-application` ablation example executes the real five-Ruvnet composition, real Cedar allow decision, and real OpenDAL memory write/read. It emits combined, Ruvnet-only, and enterprise-only rows, then one `without_*` row for each of the seven ingredients with the measured upstream score contribution where available and an explicit capability loss.

## Alternatives considered

Dependency counts, version commands, and fabricated latency claims were rejected. An ablation marked successful while omitting required product safety was rejected; removal rows are explicitly non-deployable failures.

## Consequences

The benchmark measures deterministic fixture composition, not field efficacy or statistically rigorous performance. It is reproducible and source-bound.

## Verification and mappings

Run `scripts/benchmark.sh`; verify ten rows, combined execution count seven, both baselines, and every named ingredient removal. Reciprocal mappings: `cr-application`, `cr-upstream-compose`; DDD-08.
