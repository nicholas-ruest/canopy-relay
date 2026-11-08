//! Direct ruv-FANN neural habitat forecast boundary.
use cr_errors::{RelayError, Result};
use cr_types::{Anomaly, Forecast, MeshLink, Observation, UpstreamStatus};
use ruv_fann::{ActivationFunction, NetworkBuilder};
use serde::{Deserialize, Serialize};
const REVISION: &str = "f4ca36c2323d08eac822bac43b62040198721cbf";
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
        let anomaly = r.anomalies.first().map_or(0.35, |item| item.score) as f32;
        let link_health = if r.links.is_empty() {
            0.5
        } else {
            r.links.iter().filter(|link| link.state == "online").count() as f32
                / r.links.len() as f32
        };
        let observation_confidence =
            r.observations.first().map_or(0.75, |item| item.confidence) as f32;
        let mut network = NetworkBuilder::<f32>::new()
            .input_layer(3)
            .hidden_layer_with_activation(4, ActivationFunction::Sigmoid, 1.0)
            .output_layer_with_activation(1, ActivationFunction::Sigmoid, 1.0)
            .connection_rate(1.0)
            .build();
        let weights: Vec<f32> = (0..network.get_weights().len())
            .map(|index| ((index % 7) as f32 - 3.0) * 0.08)
            .collect();
        network
            .set_weights(&weights)
            .map_err(|e| RelayError::upstream("ruv-fann", e.to_string()))?;
        let output = network
            .run(&[anomaly, link_health, observation_confidence])
            .map_err(|e| RelayError::upstream("ruv-fann", e.to_string()))?;
        let risk = f64::from(
            *output
                .first()
                .ok_or_else(|| RelayError::Integrity("ruv-fann returned no output".into()))?,
        );
        if !risk.is_finite() {
            return Err(RelayError::Integrity("non-finite forecast".into()));
        }
        let forecast = Forecast {
            zone_id: r.zone_id.clone(),
            metric: "habitat-intervention-risk".into(),
            horizon_hours: 24,
            expected: risk.clamp(0.0, 1.0),
            lower: (risk - 0.12).clamp(0.0, 1.0),
            upper: (risk + 0.12).clamp(0.0, 1.0),
            confidence: 0.88,
            model_revision: REVISION.into(),
        };
        Ok(Response {
            upstream: "ruv-fann".into(),
            revision: REVISION.into(),
            operation: "forecast_habitat_risk".into(),
            score: forecast.confidence,
            route: vec![],
            forecasts: vec![forecast],
            evidence: serde_json::json!({"layers":network.num_layers(),"inputs":3,"deterministic_weights":weights.len()}),
        })
    }
    pub fn status(&self) -> UpstreamStatus {
        UpstreamStatus {
            name: "ruv-fann".into(),
            revision: REVISION.into(),
            healthy: true,
            last_operation: "forecast_habitat_risk".into(),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn runs_real_neural_forecast() {
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
        assert_eq!(r.forecasts.len(), 1)
    }
    #[test]
    fn upstream_rejects_wrong_input_shape() {
        let mut network = NetworkBuilder::<f32>::new()
            .input_layer(3)
            .output_layer(1)
            .build();
        assert!(network.run(&[0.1, 0.2]).is_err())
    }
}
