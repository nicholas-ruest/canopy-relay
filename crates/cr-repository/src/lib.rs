use cr_errors::{RelayError, Result};
use cr_opendal_store::ArtifactStore;
use cr_types::{AuditEntry, Intervention, Observation};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::Arc};
#[derive(Clone)]
pub struct Repository {
    namespace: String,
    store: ArtifactStore,
    observations: Arc<RwLock<BTreeMap<String, Observation>>>,
    interventions: Arc<RwLock<BTreeMap<String, Intervention>>>,
    audit: Arc<RwLock<Vec<AuditEntry>>>,
}
impl Repository {
    pub fn new(ns: &str, store: ArtifactStore) -> Result<Self> {
        if ns.is_empty() {
            return Err(RelayError::Validation("namespace".into()));
        }
        Ok(Self {
            namespace: ns.into(),
            store,
            observations: Default::default(),
            interventions: Default::default(),
            audit: Default::default(),
        })
    }
    pub async fn restore(ns: &str, store: ArtifactStore) -> Result<Self> {
        let r = Self::new(ns, store.clone())?;
        for path in store.list(ns, "interventions").await? {
            let Some(id) = object_id(&path) else { continue };
            let v = store.get(ns, "interventions", id).await?;
            r.interventions.write().insert(id.into(), v);
        }
        for path in store.list(ns, "observations").await? {
            let Some(id) = object_id(&path) else { continue };
            let v = store.get(ns, "observations", id).await?;
            r.observations.write().insert(id.into(), v);
        }
        for path in store.list(ns, "audit").await? {
            let Some(id) = object_id(&path) else { continue };
            let v = store.get(ns, "audit", id).await?;
            r.audit.write().push(v)
        }
        r.audit.write().sort_by_key(|e| e.sequence);
        verify_audit_chain(&r.audit.read())?;
        Ok(r)
    }
    pub async fn save_observation(&self, v: Observation) -> Result<()> {
        self.store
            .put(&self.namespace, "observations", &v.id, &v)
            .await?;
        self.observations.write().insert(v.id.clone(), v);
        Ok(())
    }
    pub async fn save_intervention(&self, v: Intervention) -> Result<()> {
        self.store
            .put(&self.namespace, "interventions", &v.id, &v)
            .await?;
        self.interventions.write().insert(v.id.clone(), v);
        Ok(())
    }
    pub async fn append_audit(&self, v: AuditEntry) -> Result<()> {
        let expected = self.audit.read().last().map_or(1, |x| x.sequence + 1);
        if v.sequence != expected {
            return Err(RelayError::Conflict("audit gap".into()));
        }
        self.store
            .put(
                &self.namespace,
                "audit",
                &format!("{:020}-{}", v.sequence, v.id),
                &v,
            )
            .await?;
        self.audit.write().push(v);
        Ok(())
    }
    pub fn interventions(&self) -> Vec<Intervention> {
        self.interventions.read().values().cloned().collect()
    }
    pub fn audit(&self) -> Vec<AuditEntry> {
        self.audit.read().clone()
    }
    pub fn audit_count(&self) -> usize {
        self.audit.read().len()
    }
}

fn object_id(path: &str) -> Option<&str> {
    path.rsplit('/')
        .next()?
        .strip_suffix(".json")
        .filter(|id| !id.is_empty())
}

fn verify_audit_chain(entries: &[AuditEntry]) -> Result<()> {
    let mut previous = "GENESIS".to_string();
    for (index, entry) in entries.iter().enumerate() {
        let expected_sequence = index as u64 + 1;
        if entry.sequence != expected_sequence || entry.previous_hash != previous {
            return Err(RelayError::Integrity(format!(
                "audit chain discontinuity at sequence {}",
                entry.sequence
            )));
        }
        let mut digest = Sha256::new();
        digest.update(entry.sequence.to_le_bytes());
        digest.update(entry.event_type.as_bytes());
        digest.update(entry.subject_id.as_bytes());
        digest.update(entry.correlation_id.as_bytes());
        digest.update(entry.previous_hash.as_bytes());
        digest.update(
            serde_json::to_vec(&entry.payload)
                .map_err(|error| RelayError::Integrity(error.to_string()))?,
        );
        let expected_hash = hex::encode(digest.finalize());
        if entry.hash != expected_hash {
            return Err(RelayError::Integrity(format!(
                "audit hash mismatch at sequence {}",
                entry.sequence
            )));
        }
        previous.clone_from(&entry.hash);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{object_id, verify_audit_chain};
    use chrono::Utc;
    use cr_types::AuditEntry;
    use sha2::{Digest, Sha256};

    #[test]
    fn restore_ignores_directory_entries_and_accepts_objects() {
        assert_eq!(object_id("canopy-relay/audit/"), None);
        assert_eq!(
            object_id("canopy-relay/audit/0001-audit.json"),
            Some("0001-audit")
        );
    }

    fn audit(sequence: u64, previous_hash: &str, payload: serde_json::Value) -> AuditEntry {
        let mut digest = Sha256::new();
        digest.update(sequence.to_le_bytes());
        digest.update(b"event");
        digest.update(b"subject");
        digest.update(b"correlation");
        digest.update(previous_hash.as_bytes());
        digest.update(serde_json::to_vec(&payload).unwrap());
        AuditEntry {
            id: format!("audit-{sequence}"),
            sequence,
            event_type: "event".into(),
            subject_id: "subject".into(),
            correlation_id: "correlation".into(),
            payload,
            previous_hash: previous_hash.into(),
            hash: hex::encode(digest.finalize()),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn audit_chain_accepts_ordered_links_and_rejects_tampering() {
        let first = audit(1, "GENESIS", serde_json::json!({"ok": true}));
        let second = audit(2, &first.hash, serde_json::json!({"ok": false}));
        verify_audit_chain(&[first.clone(), second.clone()]).unwrap();
        let mut tampered = second;
        tampered.payload = serde_json::json!({"edited": true});
        assert!(verify_audit_chain(&[first, tampered]).is_err());
    }
}
