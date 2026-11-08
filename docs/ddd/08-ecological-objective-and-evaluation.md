# DDD-08: composition score and ingredient evaluation

## Value object

`CompositionResult` holds the route, forecasts, five named scores, and five upstream evidence objects. Its combined score is the arithmetic mean of RuCelium quality, WorldGraph graph presence, Midstream similarity, LatentMesh transport efficiency, and ruv-FANN confidence.

## Invariants

- All five upstream calls must complete in the defined order.
- LatentMesh must return a non-empty route and ruv-FANN a forecast.
- No component may be non-finite.
- The combined value informs predicted effect but never bypasses policy.

## Evaluation model

The benchmark executes the five upstreams plus Cedar and OpenDAL over the same seed. Combined, Ruvnet-only, and enterprise-only rows describe available capability; seven removal rows are marked unsuccessful and name the lost responsibility.

## Limitations

The score mixes different normalized semantics with equal weight and is not a scientific utility function. The benchmark is a reproducible integration evaluation, not a field efficacy study.

## Verification and reciprocal mappings

Composition tests, application workflow, benchmark parser, and seven ablation rows cover the context. ADR-0009 and ADR-0021 map here; owners are `cr-upstream-compose` and `cr-application`.
