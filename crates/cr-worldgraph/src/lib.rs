//! Direct WorldGraph spatial digital-twin boundary.
use cr_errors::{RelayError, Result};
use cr_types::{Anomaly, Forecast, MeshLink, Observation, UpstreamStatus};
use serde::{Deserialize, Serialize};
use wifi_densepose_geo::GeoRegistration;
use wifi_densepose_worldgraph::{WorldGraph, WorldId, WorldNode, ZoneBoundsEnu};
const REVISION: &str = "9b1c79c836cdacfb7b44f058c593157bac4c1dab";
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
    pub async fn execute(&self, request: &Request) -> Result<Response> {
        if request.correlation_id.is_empty() || request.zone_id.is_empty() {
            return Err(RelayError::Validation(
                "correlation and zone required".into(),
            ));
        }
        let mut graph = WorldGraph::new(GeoRegistration::default());
        graph.upsert_node(WorldNode::Room {
            id: WorldId(stable_id(&request.zone_id)),
            area_id: Some(request.zone_id.clone()),
            name: request.zone_id.clone(),
            bounds_enu: ZoneBoundsEnu::Rectangle {
                min_e: 0.0,
                min_n: 0.0,
                max_e: 650.0,
                max_n: 420.0,
            },
            floor: 0,
        });
        let encoded = graph
            .to_json()
            .map_err(|error| RelayError::upstream("worldgraph", error.to_string()))?;
        Ok(Response {
            upstream: "worldgraph".into(),
            revision: REVISION.into(),
            operation: "construct_habitat_graph".into(),
            score: (graph.node_count() as f64 / 4.0).clamp(0.0, 1.0),
            route: vec![],
            forecasts: vec![],
            evidence: serde_json::json!({"nodes":graph.node_count(),"snapshot_bytes":encoded.len(),"zone":request.zone_id}),
        })
    }
    pub fn status(&self) -> UpstreamStatus {
        UpstreamStatus {
            name: "worldgraph".into(),
            revision: REVISION.into(),
            healthy: true,
            last_operation: "construct_habitat_graph".into(),
        }
    }
}
fn stable_id(value: &str) -> u64 {
    value
        .bytes()
        .fold(17_u64, |acc, byte| {
            acc.wrapping_mul(37).wrapping_add(u64::from(byte))
        })
        .max(1)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn constructs_real_graph() {
        let r = Client::new()
            .execute(&Request {
                correlation_id: "c".into(),
                zone_id: "ravine".into(),
                observations: vec![],
                anomalies: vec![],
                links: vec![],
            })
            .await
            .unwrap();
        assert_eq!(r.evidence["nodes"], 1)
    }
    #[tokio::test]
    async fn empty_zone_is_rejected() {
        assert!(Client::new()
            .execute(&Request {
                correlation_id: "c".into(),
                zone_id: "".into(),
                observations: vec![],
                anomalies: vec![],
                links: vec![]
            })
            .await
            .is_err())
    }
}
