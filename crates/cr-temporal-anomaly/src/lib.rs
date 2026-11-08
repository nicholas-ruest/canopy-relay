//! robust anomaly windows.
use cr_errors::{RelayError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TemporalDetectorRecord {
    pub id: String,
    pub subject: String,
    pub kind: String,
    pub value: f64,
    pub parents: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub digest: String,
}
#[derive(Default)]
pub struct TemporalDetector {
    records: BTreeMap<String, TemporalDetectorRecord>,
    roots: BTreeSet<String>,
}
impl TemporalDetector {
    pub fn append(
        &mut self,
        id: &str,
        subject: &str,
        kind: &str,
        value: f64,
        parents: Vec<String>,
        metadata: BTreeMap<String, String>,
    ) -> Result<TemporalDetectorRecord> {
        if id.trim().is_empty()
            || subject.trim().is_empty()
            || kind.trim().is_empty()
            || !value.is_finite()
        {
            return Err(RelayError::Validation("invalid sample".into()));
        }
        if self.records.contains_key(id) {
            return Err(RelayError::Conflict(format!("duplicate sample {id}")));
        }
        if parents.iter().any(|p| !self.records.contains_key(p)) {
            return Err(RelayError::NotFound("missing parent sample".into()));
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
        let r = TemporalDetectorRecord {
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
    pub fn get(&self, id: &str) -> Result<&TemporalDetectorRecord> {
        self.records
            .get(id)
            .ok_or_else(|| RelayError::NotFound(id.into()))
    }
    pub fn lineage(&self, id: &str) -> Result<Vec<&TemporalDetectorRecord>> {
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
    fn window_keeps_lineage() {
        let mut b = TemporalDetector::default();
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
        let mut b = TemporalDetector::default();
        assert!(b
            .append("b", "z", "bad", 1.0, vec!["x".into()], BTreeMap::new())
            .is_err())
    }
}

/// Robust rolling detector using median absolute deviation and hysteresis.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RobustDetector {
    capacity: usize,
    enter_threshold: f64,
    exit_threshold: f64,
    values: Vec<f64>,
    active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AnomalyDecision {
    pub value: f64,
    pub baseline: f64,
    pub deviation: f64,
    pub score: f64,
    pub active: bool,
}

impl RobustDetector {
    pub fn new(capacity: usize, enter_threshold: f64, exit_threshold: f64) -> Result<Self> {
        if capacity < 5
            || !enter_threshold.is_finite()
            || !exit_threshold.is_finite()
            || enter_threshold <= exit_threshold
            || exit_threshold <= 0.0
        {
            return Err(RelayError::Validation(
                "invalid robust detector configuration".into(),
            ));
        }
        Ok(Self {
            capacity,
            enter_threshold,
            exit_threshold,
            values: vec![],
            active: false,
        })
    }

    pub fn observe(&mut self, value: f64) -> Result<AnomalyDecision> {
        if !value.is_finite() {
            return Err(RelayError::Validation("non-finite temporal value".into()));
        }
        let baseline = median(&self.values).unwrap_or(value);
        let deviations: Vec<f64> = self
            .values
            .iter()
            .map(|candidate| (candidate - baseline).abs())
            .collect();
        let mad = median(&deviations).unwrap_or(0.0).max(1.0e-6);
        let deviation = (value - baseline).abs();
        let score = deviation / (1.4826 * mad);
        self.active = if self.active {
            score >= self.exit_threshold
        } else {
            score >= self.enter_threshold && self.values.len() >= 5
        };
        self.values.push(value);
        if self.values.len() > self.capacity {
            self.values.remove(0);
        }
        Ok(AnomalyDecision {
            value,
            baseline,
            deviation,
            score,
            active: self.active,
        })
    }
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    })
}

#[cfg(test)]
mod detector_tests {
    use super::*;
    #[test]
    fn spike_enters_and_recovery_exits_alert() {
        let mut detector = RobustDetector::new(12, 5.0, 2.0).unwrap();
        for value in [10.0, 10.1, 9.9, 10.0, 10.05, 9.95] {
            assert!(!detector.observe(value).unwrap().active);
        }
        assert!(detector.observe(30.0).unwrap().active);
        assert!(!detector.observe(10.0).unwrap().active);
    }
    #[test]
    fn invalid_samples_and_thresholds_fail() {
        assert!(RobustDetector::new(3, 2.0, 3.0).is_err());
        let mut detector = RobustDetector::new(5, 4.0, 2.0).unwrap();
        assert!(detector.observe(f64::NAN).is_err());
    }
}
