use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}
impl GeoPoint {
    pub fn valid(&self) -> bool {
        (-90.0..=90.0).contains(&self.lat) && (-180.0..=180.0).contains(&self.lon)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ObservationInput {
    pub device_id: String,
    pub zone_id: String,
    pub species: String,
    pub metric: String,
    pub value: f64,
    pub unit: String,
    pub captured_at: DateTime<Utc>,
    pub location: GeoPoint,
    pub calibration_version: String,
    pub nonce: String,
    pub signature: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Observation {
    pub id: String,
    pub source: ObservationInput,
    pub calibrated_value: f64,
    pub confidence: f64,
    pub provenance_hash: String,
    pub correlation_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HabitatZone {
    pub id: String,
    pub name: String,
    pub centroid: GeoPoint,
    pub habitat_class: String,
    pub sensitivity: String,
    pub neighbors: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SensorSummary {
    pub device_id: String,
    pub zone_id: String,
    pub trust_state: String,
    pub last_seen: DateTime<Utc>,
    pub calibration_version: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Anomaly {
    pub id: String,
    pub zone_id: String,
    pub metric: String,
    pub score: f64,
    pub observed: f64,
    pub expected: f64,
    pub evidence_ids: Vec<String>,
    pub state: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MeshLink {
    pub from_zone: String,
    pub to_zone: String,
    pub capacity: f64,
    pub ecological_cost: f64,
    pub latency_ms: u64,
    pub state: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Forecast {
    pub zone_id: String,
    pub metric: String,
    pub horizon_hours: u32,
    pub expected: f64,
    pub lower: f64,
    pub upper: f64,
    pub confidence: f64,
    pub model_revision: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InterventionRequest {
    pub anomaly_id: String,
    pub action: String,
    pub zone_id: String,
    pub rationale: String,
    pub parameters: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Intervention {
    pub id: String,
    pub request: InterventionRequest,
    pub state: String,
    pub route: Vec<String>,
    pub predicted_effect: f64,
    pub policy_decision: String,
    pub correlation_id: String,
    pub created_at: DateTime<Utc>,
    pub evidence_hash: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UpstreamStatus {
    pub name: String,
    pub revision: String,
    pub healthy: bool,
    pub last_operation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    pub id: String,
    pub sequence: u64,
    pub event_type: String,
    pub subject_id: String,
    pub correlation_id: String,
    pub payload: serde_json::Value,
    pub previous_hash: String,
    pub hash: String,
    pub created_at: DateTime<Utc>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dashboard {
    pub zones: Vec<HabitatZone>,
    pub sensors: Vec<SensorSummary>,
    pub anomalies: Vec<Anomaly>,
    pub interventions: Vec<Intervention>,
    pub mesh_links: Vec<MeshLink>,
    pub audit: Vec<AuditEntry>,
    pub upstreams: Vec<UpstreamStatus>,
    pub generated_at: DateTime<Utc>,
}
pub fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn location_bounds() {
        assert!(GeoPoint {
            lat: 43.0,
            lon: -79.0
        }
        .valid());
        assert!(!GeoPoint {
            lat: 143.0,
            lon: -79.0
        }
        .valid())
    }
    #[test]
    fn ids_are_domain_scoped() {
        assert!(new_id("obs").starts_with("obs-"))
    }
}
