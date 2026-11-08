//! causal source lineage.
use cr_errors::{RelayError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProvenanceLedgerRecord {
    pub id: String,
    pub subject: String,
    pub kind: String,
    pub value: f64,
    pub parents: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub digest: String,
}
#[derive(Default)]
pub struct ProvenanceLedger {
    records: BTreeMap<String, ProvenanceLedgerRecord>,
    roots: BTreeSet<String>,
}
impl ProvenanceLedger {
    pub fn append(
        &mut self,
        id: &str,
        subject: &str,
        kind: &str,
        value: f64,
        parents: Vec<String>,
        metadata: BTreeMap<String, String>,
    ) -> Result<ProvenanceLedgerRecord> {
        if id.trim().is_empty()
            || subject.trim().is_empty()
            || kind.trim().is_empty()
            || !value.is_finite()
        {
            return Err(RelayError::Validation("invalid claim".into()));
        }
        if self.records.contains_key(id) {
            return Err(RelayError::Conflict(format!("duplicate claim {id}")));
        }
        if parents.iter().any(|p| !self.records.contains_key(p)) {
            return Err(RelayError::NotFound("missing parent claim".into()));
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
        let r = ProvenanceLedgerRecord {
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
    pub fn get(&self, id: &str) -> Result<&ProvenanceLedgerRecord> {
        self.records
            .get(id)
            .ok_or_else(|| RelayError::NotFound(id.into()))
    }
    pub fn lineage(&self, id: &str) -> Result<Vec<&ProvenanceLedgerRecord>> {
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
    fn source_keeps_lineage() {
        let mut b = ProvenanceLedger::default();
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
        let mut b = ProvenanceLedger::default();
        assert!(b
            .append("b", "z", "bad", 1.0, vec!["x".into()], BTreeMap::new())
            .is_err())
    }
}

/// Append-only evidence chain used to detect reordered or edited field facts.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceLink {
    pub sequence: u64,
    pub subject: String,
    pub payload_hash: String,
    pub previous_hash: String,
    pub chain_hash: String,
}

#[derive(Default)]
pub struct EvidenceChain {
    links: Vec<EvidenceLink>,
}

impl EvidenceChain {
    pub fn append(&mut self, subject: &str, payload: &[u8]) -> Result<EvidenceLink> {
        if subject.is_empty() || payload.is_empty() {
            return Err(RelayError::Validation(
                "provenance subject and payload required".into(),
            ));
        }
        let payload_hash = hex::encode(Sha256::digest(payload));
        let previous_hash = self
            .links
            .last()
            .map_or_else(|| "genesis".into(), |link| link.chain_hash.clone());
        let sequence = self.links.len() as u64 + 1;
        let chain_hash = hash_link(sequence, subject, &payload_hash, &previous_hash);
        let link = EvidenceLink {
            sequence,
            subject: subject.into(),
            payload_hash,
            previous_hash,
            chain_hash,
        };
        self.links.push(link.clone());
        Ok(link)
    }

    pub fn verify(&self) -> Result<()> {
        let mut previous = "genesis".to_string();
        for (index, link) in self.links.iter().enumerate() {
            if link.sequence != index as u64 + 1
                || link.previous_hash != previous
                || link.chain_hash
                    != hash_link(
                        link.sequence,
                        &link.subject,
                        &link.payload_hash,
                        &link.previous_hash,
                    )
            {
                return Err(RelayError::Integrity(format!(
                    "broken provenance link {}",
                    link.sequence
                )));
            }
            previous.clone_from(&link.chain_hash);
        }
        Ok(())
    }

    pub fn links(&self) -> &[EvidenceLink] {
        &self.links
    }
}

fn hash_link(sequence: u64, subject: &str, payload: &str, previous: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(sequence.to_le_bytes());
    digest.update(subject);
    digest.update(payload);
    digest.update(previous);
    hex::encode(digest.finalize())
}

#[cfg(test)]
mod chain_tests {
    use super::*;
    #[test]
    fn ordered_chain_verifies() {
        let mut chain = EvidenceChain::default();
        chain.append("observation", b"soil=0.44").unwrap();
        chain.append("forecast", b"risk=0.62").unwrap();
        chain.verify().unwrap();
        assert_eq!(chain.links()[1].previous_hash, chain.links()[0].chain_hash);
    }
    #[test]
    fn edited_chain_is_detected() {
        let mut chain = EvidenceChain::default();
        chain.append("observation", b"soil=0.44").unwrap();
        chain.links[0].subject = "edited".into();
        assert!(chain.verify().is_err());
    }
}
