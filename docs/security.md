# Security and privacy

Sensitive assets include device signing identity, protected-species coordinates, raw acoustic samples, steward authorization, intervention schedules, and audit integrity. Threats include forged or replayed batches, malicious geometry, cross-site access, location inference, policy bypass, partial writes, audit truncation, dependency timeout, and overconfident output.

Controls:

- a real RuCelium signed-sample boundary plus provisioned devices and one-use 64-hex observation receipts;
- monotonic registry sequences and repeated receipt digests fail closed;
- only invented coarse coordinates are stored; no raw acoustic content or protected-species locations enter the fixture;
- authentication and route role checks precede domain work; hard safety exclusion and Cedar execute before selected upstreams or writes;
- validated correlation IDs and timeout-bounded upstream/handler work;
- typed failures that cannot become nominal forecasts;
- ordered intervention then hash-linked audit writes, with exact audit verification on reopen;
- logs exclude bearer material, raw samples, exact sensitive coordinates, and field-note bodies.

Local bearer auth and filesystem-backed OpenDAL are prototype mechanisms, not production IAM or multi-region storage. Production requires managed identity, key rotation, encrypted shared storage, tenant isolation, backups, retention governance, and incident response. Canopy Relay must never trigger irreversible ecological action automatically.

Required negative tests include tampered RuCelium probe, receipt replay, wrong role, non-JSON mutation, malformed location, restricted-action denial, wrong neural input shape, timeout, unsafe storage key, and audit corruption. Exact-source results live in the external artifact rather than this document.

