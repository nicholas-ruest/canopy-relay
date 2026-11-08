# DDD-02: observation trust

## Aggregate

`TrustRegistry` is the aggregate root for provisioned field devices. A device has firmware and signer identities, a revocation flag, and monotonic sequence state. The browser-facing path also admits one-use signed-ingest receipt digests issued by the field boundary.

## Invariants

- Only provisioned and non-revoked devices may submit receipts.
- A receipt is exactly 64 hexadecimal characters and is accepted once.
- A lower or repeated device sequence conflicts with prior admission.
- Trust rejection occurs before observation, repository, or audit mutation.

## Commands and failures

`provision`, `admit`, `admit_receipt`, and `revoke` are explicit commands. Unknown device and mismatched attestation are unauthenticated; revoked device is denied; replay is a conflict. These distinctions become typed API errors.

## Upstream relationship

`cr-rucelium` independently demonstrates a real signed sample, registered public key, and verified `IngestPipeline` result during the composed product workflow. Canopy Relay does not claim its local prototype receipt format is RuCelium's wire envelope.

## Verification and reciprocal mappings

Registry behavior/failure tests, real RuCelium tests, and application replay acceptance cover this context. ADR-0003 and ADR-0019 map here; owners are `cr-device-trust`, `cr-rucelium`, and `cr-application`.
