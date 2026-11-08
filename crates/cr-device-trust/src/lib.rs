//! signed device attestations.
use cr_errors::{RelayError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeviceTrustRecord {
    pub id: String,
    pub subject: String,
    pub kind: String,
    pub value: f64,
    pub parents: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub digest: String,
}
#[derive(Default)]
pub struct DeviceTrust {
    records: BTreeMap<String, DeviceTrustRecord>,
    roots: BTreeSet<String>,
}
impl DeviceTrust {
    pub fn append(
        &mut self,
        id: &str,
        subject: &str,
        kind: &str,
        value: f64,
        parents: Vec<String>,
        metadata: BTreeMap<String, String>,
    ) -> Result<DeviceTrustRecord> {
        if id.trim().is_empty()
            || subject.trim().is_empty()
            || kind.trim().is_empty()
            || !value.is_finite()
        {
            return Err(RelayError::Validation("invalid device".into()));
        }
        if self.records.contains_key(id) {
            return Err(RelayError::Conflict(format!("duplicate device {id}")));
        }
        if parents.iter().any(|p| !self.records.contains_key(p)) {
            return Err(RelayError::NotFound("missing parent device".into()));
        }
        if parents.iter().any(|p| p == id) {
            return Err(RelayError::Validation("cycle".into()));
        }
        let mut h = Sha256::new();
        h.update(id);
        h.update(subject);
        h.update(kind);
        h.update(value.to_le_bytes());
        for p in &parents {
            h.update(p)
        }
        for (k, v) in &metadata {
            h.update(k);
            h.update(v)
        }
        let r = DeviceTrustRecord {
            id: id.into(),
            subject: subject.into(),
            kind: kind.into(),
            value,
            parents: parents.clone(),
            metadata,
            digest: hex::encode(h.finalize()),
        };
        if parents.is_empty() {
            self.roots.insert(id.into());
        }
        self.records.insert(id.into(), r.clone());
        Ok(r)
    }
    pub fn get(&self, id: &str) -> Result<&DeviceTrustRecord> {
        self.records
            .get(id)
            .ok_or_else(|| RelayError::NotFound(id.into()))
    }
    pub fn lineage(&self, id: &str) -> Result<Vec<&DeviceTrustRecord>> {
        let mut out = vec![];
        let mut pending = vec![id.to_string()];
        let mut seen = BTreeSet::new();
        while let Some(next) = pending.pop() {
            if !seen.insert(next.clone()) {
                continue;
            }
            let r = self.get(&next)?;
            pending.extend(r.parents.iter().cloned());
            out.push(r)
        }
        Ok(out)
    }
    pub fn aggregate(&self, subject: &str) -> Result<f64> {
        let v: Vec<_> = self
            .records
            .values()
            .filter(|r| r.subject == subject)
            .map(|r| r.value)
            .collect();
        if v.is_empty() {
            return Err(RelayError::NotFound(subject.into()));
        }
        Ok(v.iter().sum::<f64>() / v.len() as f64)
    }
    pub fn len(&self) -> usize {
        self.records.len()
    }
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn key_keeps_lineage() {
        let mut b = DeviceTrust::default();
        b.append("a", "zone", "baseline", 1.0, vec![], BTreeMap::new())
            .unwrap();
        b.append(
            "b",
            "zone",
            "derived",
            2.0,
            vec!["a".into()],
            BTreeMap::new(),
        )
        .unwrap();
        assert_eq!(b.lineage("b").unwrap().len(), 2);
        assert_eq!(b.aggregate("zone").unwrap(), 1.5)
    }
    #[test]
    fn missing_parent_fails() {
        let mut b = DeviceTrust::default();
        assert!(b
            .append("b", "z", "bad", 1.0, vec!["x".into()], BTreeMap::new())
            .is_err())
    }
}

/// Provisioning state used before a signed observation can reach ingest.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvisionedDevice {
    pub device_id: String,
    pub firmware_hash: String,
    pub signer_hash: String,
    pub last_sequence: Option<u64>,
    pub revoked: bool,
}

/// A replay-safe field-device registry with explicit re-provisioning semantics.
#[derive(Default)]
pub struct TrustRegistry {
    devices: BTreeMap<String, ProvisionedDevice>,
    admitted_receipts: BTreeSet<String>,
}

impl TrustRegistry {
    pub fn provision(
        &mut self,
        device_id: &str,
        firmware_hash: &str,
        signer_hash: &str,
    ) -> Result<()> {
        if [device_id, firmware_hash, signer_hash]
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(RelayError::Validation(
                "device, firmware, and signer are required".into(),
            ));
        }
        self.devices.insert(
            device_id.into(),
            ProvisionedDevice {
                device_id: device_id.into(),
                firmware_hash: firmware_hash.into(),
                signer_hash: signer_hash.into(),
                last_sequence: None,
                revoked: false,
            },
        );
        Ok(())
    }

    pub fn admit(
        &mut self,
        device_id: &str,
        firmware_hash: &str,
        signer_hash: &str,
        sequence: u64,
    ) -> Result<String> {
        let device = self
            .devices
            .get_mut(device_id)
            .ok_or_else(|| RelayError::Unauthenticated("unprovisioned device".into()))?;
        if device.revoked {
            return Err(RelayError::Denied("device is revoked".into()));
        }
        if device.firmware_hash != firmware_hash || device.signer_hash != signer_hash {
            return Err(RelayError::Unauthenticated(
                "attestation does not match provisioning".into(),
            ));
        }
        if device
            .last_sequence
            .is_some_and(|previous| sequence <= previous)
        {
            return Err(RelayError::Conflict("device sequence replay".into()));
        }
        device.last_sequence = Some(sequence);
        let mut digest = Sha256::new();
        digest.update(device_id);
        digest.update(firmware_hash);
        digest.update(signer_hash);
        digest.update(sequence.to_le_bytes());
        Ok(hex::encode(digest.finalize()))
    }

    pub fn revoke(&mut self, device_id: &str) -> Result<()> {
        let device = self
            .devices
            .get_mut(device_id)
            .ok_or_else(|| RelayError::NotFound(device_id.into()))?;
        device.revoked = true;
        Ok(())
    }

    /// Admit a source-bound receipt issued by the signed-ingest boundary.
    /// Receipts are one-time capabilities: replay fails before any durable
    /// observation or audit effect can occur.
    pub fn admit_receipt(&mut self, device_id: &str, receipt: &str) -> Result<()> {
        let device = self
            .devices
            .get(device_id)
            .ok_or_else(|| RelayError::Unauthenticated("unprovisioned device".into()))?;
        if device.revoked {
            return Err(RelayError::Denied("device is revoked".into()));
        }
        if receipt.len() != 64 || !receipt.bytes().all(|value| value.is_ascii_hexdigit()) {
            return Err(RelayError::Unauthenticated(
                "signed-ingest receipt must be a 64-character hex digest".into(),
            ));
        }
        if !self.admitted_receipts.insert(receipt.to_ascii_lowercase()) {
            return Err(RelayError::Conflict("device trust receipt replay".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod registry_tests {
    use super::*;
    #[test]
    fn provisioning_admits_monotonic_attestations() {
        let mut registry = TrustRegistry::default();
        registry.provision("sensor-7", "fw-a", "key-a").unwrap();
        assert_eq!(
            registry
                .admit("sensor-7", "fw-a", "key-a", 1)
                .unwrap()
                .len(),
            64
        );
        assert!(registry.admit("sensor-7", "fw-a", "key-a", 1).is_err());
    }
    #[test]
    fn revoked_or_mismatched_devices_fail_closed() {
        let mut registry = TrustRegistry::default();
        registry.provision("sensor-7", "fw-a", "key-a").unwrap();
        assert!(registry.admit("sensor-7", "fw-b", "key-a", 1).is_err());
        registry.revoke("sensor-7").unwrap();
        assert!(registry.admit("sensor-7", "fw-a", "key-a", 2).is_err());
    }

    #[test]
    fn signed_receipts_are_single_use_and_require_provisioning() {
        let receipt = "a".repeat(64);
        let mut registry = TrustRegistry::default();
        assert!(registry.admit_receipt("sensor-7", &receipt).is_err());
        registry.provision("sensor-7", "fw-a", "key-a").unwrap();
        registry.admit_receipt("sensor-7", &receipt).unwrap();
        assert!(registry.admit_receipt("sensor-7", &receipt).is_err());
        assert!(registry.admit_receipt("sensor-7", "not-a-digest").is_err());
    }
}
