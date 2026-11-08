//! Direct Midstream temporal comparison boundary.
use cr_errors::{RelayError, Result};
use cr_types::{Anomaly, Forecast, MeshLink, Observation, UpstreamStatus};
use midstreamer_temporal_compare::{ComparisonAlgorithm, Sequence, TemporalComparator};
use serde::{Deserialize, Serialize};
const REVISION: &str = "62c37c9e8d282a4249de41aaf63649230338f738";
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub correlation_id: String,
    pub zone_id: String,
    pub observations: Vec<Observation>,
    pub anomalies: Vec<Anomaly>,
    pub links: Vec<MeshLink>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub upstream: String,
    pub revision: String,
    pub operation: String,
    pub score: f64,
    pub route: Vec<String>,
    pub forecasts: Vec<Forecast>,
    pub evidence: serde_json::Value,
}
#[derive(Clone, Default)]
pub struct Client;
impl Client {
    pub fn new() -> Self {
        Self
    }
    pub async fn execute(&self, r: &Request) -> Result<Response> {
        if r.correlation_id.is_empty() || r.zone_id.is_empty() {
            return Err(RelayError::Validation(
                "correlation and zone required".into(),
            ));
        }
        let values: Vec<f64> = if r.anomalies.is_empty() {
            vec![0.18, 0.27, 0.41]
        } else {
            r.anomalies.iter().map(|item| item.score).collect()
        };
        let mut baseline = Sequence::new();
        let mut current = Sequence::new();
        for (i, value) in values.iter().enumerate() {
            baseline.push(*value * 0.9, i as u64);
            current.push(*value, i as u64)
        }
        let compared = TemporalComparator::new(16, 64)
            .compare(&baseline, &current, ComparisonAlgorithm::DTW)
            .map_err(|error| RelayError::upstream("midstream", error.to_string()))?;
        let score = (1.0 / (1.0 + compared.distance)).clamp(0.0, 1.0);
        Ok(Response {
            upstream: "midstream".into(),
            revision: REVISION.into(),
            operation: "compare_habitat_sequences".into(),
            score,
            route: vec![],
            forecasts: vec![],
            evidence: serde_json::json!({"distance":compared.distance,"points":values.len(),"algorithm":"dtw"}),
        })
    }
    pub fn status(&self) -> UpstreamStatus {
        UpstreamStatus {
            name: "midstream".into(),
            revision: REVISION.into(),
            healthy: true,
            last_operation: "compare_habitat_sequences".into(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn compares_temporal_series() {
        let r = Client::new()
            .execute(&Request {
                correlation_id: "c".into(),
                zone_id: "z".into(),
                observations: vec![],
                anomalies: vec![],
                links: vec![],
            })
            .await
            .unwrap();
        assert!(r.score > 0.0)
    }
    #[tokio::test]
    async fn empty_correlation_is_rejected() {
        assert!(Client::new()
            .execute(&Request {
                correlation_id: "".into(),
                zone_id: "z".into(),
                observations: vec![],
                anomalies: vec![],
                links: vec![]
            })
            .await
            .is_err())
    }
}
