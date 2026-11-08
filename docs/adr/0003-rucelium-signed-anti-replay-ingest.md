# ADR-0003: signed field evidence and one-use trust receipts

## Context

An ecological observation should not mutate durable state unless its device is provisioned and the signed-ingest boundary has issued a source-bound receipt. Replayed receipts must not create duplicate observations.

## Decision

The intervention composition executes pinned RuCelium by signing an environmental sample, registering its device key, and verifying the envelope through `IngestPipeline`. Browser observation ingest separately requires a 64-character receipt, admits it once against a provisioned seed device in `TrustRegistry`, and rejects replay before repository or audit writes.

## Alternatives considered

Header presence alone was rejected because it permits replay. Treating TLS as field provenance was rejected because offline forwarding loses the original channel. Idempotently accepting an opaque replay was rejected because this prototype does not expose a client-supplied observation identity.

## Consequences

The prototype demonstrates verification and replay control but does not claim production key rotation or remote attestation distribution. Device provisioning is deterministic seed data.

## Verification and mappings

Run real RuCelium success/failure tests plus `replayed_device_receipt_is_rejected_before_a_second_write`. Reciprocal mappings: `cr-rucelium`, `cr-device-trust`, `cr-application`; DDD-02.
