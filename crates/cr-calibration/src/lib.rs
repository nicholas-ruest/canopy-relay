//! versioned bounded calibration.
use cr_errors::{RelayError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalibrationBookRecord {
    pub id: String,
    pub subject: String,
    pub kind: String,
    pub value: f64,
    pub parents: Vec<String>,
    pub metadata: BTreeMap<String, String>,
    pub digest: String,
}
#[derive(Default)]
pub struct CalibrationBook {
    records: BTreeMap<String, CalibrationBookRecord>,
    roots: BTreeSet<String>,
}
impl CalibrationBook {
    pub fn append(
        &mut self,
        id: &str,
        subject: &str,
        kind: &str,
        value: f64,
        parents: Vec<String>,
        metadata: BTreeMap<String, String>,
    ) -> Result<CalibrationBookRecord> {
        if id.trim().is_empty()
            || subject.trim().is_empty()
            || kind.trim().is_empty()
            || !value.is_finite()
        {
            return Err(RelayError::Validation("invalid curve".into()));
        }
        if self.records.contains_key(id) {
            return Err(RelayError::Conflict(format!("duplicate curve {id}")));
        }
        if parents.iter().any(|p| !self.records.contains_key(p)) {
            return Err(RelayError::NotFound("missing parent curve".into()));
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
        let r = CalibrationBookRecord {
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
    pub fn get(&self, id: &str) -> Result<&CalibrationBookRecord> {
        self.records
            .get(id)
            .ok_or_else(|| RelayError::NotFound(id.into()))
    }
    pub fn lineage(&self, id: &str) -> Result<Vec<&CalibrationBookRecord>> {
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
    fn version_keeps_lineage() {
        let mut b = CalibrationBook::default();
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
        let mut b = CalibrationBook::default();
        assert!(b
            .append("b", "z", "bad", 1.0, vec!["x".into()], BTreeMap::new())
            .is_err())
    }
}

/// Monotonic piecewise-linear field calibration with bounded extrapolation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalibrationCurve {
    pub version: String,
    pub points: Vec<(f64, f64)>,
    pub max_extrapolation: f64,
}

impl CalibrationCurve {
    pub fn new(version: impl Into<String>, points: Vec<(f64, f64)>) -> Result<Self> {
        let version = version.into();
        if version.is_empty() || points.len() < 2 {
            return Err(RelayError::Validation(
                "calibration needs a version and two points".into(),
            ));
        }
        if points
            .iter()
            .any(|(raw, corrected)| !raw.is_finite() || !corrected.is_finite())
            || points.windows(2).any(|pair| pair[0].0 >= pair[1].0)
        {
            return Err(RelayError::Validation(
                "calibration raw values must be finite and increasing".into(),
            ));
        }
        Ok(Self {
            version,
            points,
            max_extrapolation: 0.05,
        })
    }

    pub fn apply(&self, raw: f64) -> Result<f64> {
        if !raw.is_finite() {
            return Err(RelayError::Validation("non-finite sensor reading".into()));
        }
        let first = self.points[0];
        let last = *self.points.last().expect("validated non-empty points");
        let span = last.0 - first.0;
        if raw < first.0 - span * self.max_extrapolation
            || raw > last.0 + span * self.max_extrapolation
        {
            return Err(RelayError::Validation(
                "reading exceeds calibration envelope".into(),
            ));
        }
        let segment = self
            .points
            .windows(2)
            .find(|pair| raw >= pair[0].0 && raw <= pair[1].0)
            .unwrap_or_else(|| {
                if raw < first.0 {
                    &self.points[0..2]
                } else {
                    &self.points[self.points.len() - 2..]
                }
            });
        let ratio = (raw - segment[0].0) / (segment[1].0 - segment[0].0);
        Ok(segment[0].1 + ratio * (segment[1].1 - segment[0].1))
    }

    pub fn drift_against(&self, successor: &Self, reference: f64) -> Result<f64> {
        if self.version == successor.version {
            return Err(RelayError::Conflict(
                "drift comparison requires distinct versions".into(),
            ));
        }
        Ok((self.apply(reference)? - successor.apply(reference)?).abs())
    }
}

#[cfg(test)]
mod curve_tests {
    use super::*;
    #[test]
    fn curve_interpolates_and_quantifies_drift() {
        let first = CalibrationCurve::new("v1", vec![(0.0, 0.0), (10.0, 20.0)]).unwrap();
        let next = CalibrationCurve::new("v2", vec![(0.0, 1.0), (10.0, 21.0)]).unwrap();
        assert_eq!(first.apply(5.0).unwrap(), 10.0);
        assert_eq!(first.drift_against(&next, 5.0).unwrap(), 1.0);
    }
    #[test]
    fn unsafe_extrapolation_is_rejected() {
        let curve = CalibrationCurve::new("v1", vec![(0.0, 0.0), (10.0, 20.0)]).unwrap();
        assert!(curve.apply(20.0).is_err());
        assert!(CalibrationCurve::new("bad", vec![(1.0, 1.0), (1.0, 2.0)]).is_err());
    }
}
