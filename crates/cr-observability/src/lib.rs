//! correlated dependency spans.
use cr_errors::{RelayError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TraceBookRecord {
    pub id: String,
    pub subject: String,
    pub kind: String,
    pub value: f64,
    pub parents: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub digest: String,
}
#[derive(Default)]
pub struct TraceBook {
    records: BTreeMap<String, TraceBookRecord>,
    roots: BTreeSet<String>,
}
impl TraceBook {
    pub fn append(
        &mut self,
        id: &str,
        subject: &str,
        kind: &str,
        value: f64,
        parents: Vec<String>,
        metadata: BTreeMap<String, String>,
    ) -> Result<TraceBookRecord> {
        if id.trim().is_empty()
            || subject.trim().is_empty()
            || kind.trim().is_empty()
            || !value.is_finite()
        {
            return Err(RelayError::Validation("invalid span".into()));
        }
        if self.records.contains_key(id) {
            return Err(RelayError::Conflict(format!("duplicate span {id}")));
        }
        if parents.iter().any(|p| !self.records.contains_key(p)) {
            return Err(RelayError::NotFound("missing parent span".into()));
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
        let r = TraceBookRecord {
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
    pub fn get(&self, id: &str) -> Result<&TraceBookRecord> {
        self.records
            .get(id)
            .ok_or_else(|| RelayError::NotFound(id.into()))
    }
    pub fn lineage(&self, id: &str) -> Result<Vec<&TraceBookRecord>> {
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
    fn correlation_keeps_lineage() {
        let mut b = TraceBook::default();
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
        let mut b = TraceBook::default();
        assert!(b
            .append("b", "z", "bad", 1.0, vec!["x".into()], BTreeMap::new())
            .is_err())
    }
}
