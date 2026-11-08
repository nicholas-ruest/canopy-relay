# ADR-0022: runtime evidence is external and exact-source-bound

## Context

Committed PASS text can be written without execution, and replacing a SHA in an old artifact creates false provenance. Verification also produces files that would make the checkout dirty.

## Decision

Verify one clean immutable candidate. Store recorder receipts, raw logs, Cargo metadata, composition report, semantic review, browser artifacts, correlations, benchmarks, ablations, and gate outputs in an external schema-v2 artifact with hashes. Repository files are pointers/readback receipts only.

## Alternatives considered

Committed self-attestation, circular tree hashes, mutable latest links, and rebinding old evidence were rejected. A gate output alone was rejected because structural validity is not semantic product proof.

## Consequences

Each source change requires a new run and durable artifact, increasing CI cost. Evidence remains independently downloadable and internally consistent.

## Verification and mappings

Check source SHA, run/head SHA, producer ID, argv, timestamps, exit codes, file hashes, artifact download, and semantic-review agreement. Reciprocal mappings: evidence scripts, `cr-application`; DDD-10.

