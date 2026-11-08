# ADR-0023: container topology mirrors local product; cloud files are config only

## Context

A host-only demo can depend on undeclared processes, paths, or caches. Conversely, deploying just to prove compatibility would create unauthorized spend and risk.

## Decision

Docker Compose starts the built frontend, default `cr-service` binary, and isolated OpenDAL volume with health/readiness and the same configuration contract as local scripts. Cloud Run service and rollback files remain reviewed configuration until separately authorized.

## Alternatives considered

Host-only hidden setup and a special container binary were rejected because they create two products. Automatic cloud deployment and managed-service provisioning were rejected as outside authority.

## Consequences

Container builds add time and local volumes do not prove production durability. They do prove topology, ports, readiness, and process replacement under a reproducible launch boundary.

## Verification and mappings

Build from a clean context, launch, poll readiness, run a real API operation, restart backend, retrieve state, stop, and inspect exit codes. Reciprocal mappings: `cr-service`, container definitions, delivery scripts; DDD-11.

