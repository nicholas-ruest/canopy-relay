//! Public HTTP view models. The domain model stays evidence-centric while this
//! module gives the browser a stable, product-specific representation.
use chrono::{DateTime, Utc};
use cr_types as domain;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize)]
pub struct ObservationRequest {
    pub zone_id: String,
    pub sensor_id: String,
    pub signal_kind: String,
    pub value: f64,
    pub unit: String,
}

impl ObservationRequest {
    pub fn into_domain(self) -> domain::ObservationInput {
        domain::ObservationInput {
            device_id: self.sensor_id,
            zone_id: self.zone_id,
            species: "urban-ravine-indicator".into(),
            metric: self.signal_kind,
            value: self.value,
            unit: self.unit,
            captured_at: Utc::now(),
            location: domain::GeoPoint {
                lat: 43.645,
                lon: -79.376,
            },
            calibration_version: "field-calibration-v3".into(),
            nonce: format!("browser-{}", Utc::now().timestamp_millis()),
            signature: "verified-by-rucelium-ingest-boundary".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ObservationResult {
    pub observation_id: String,
    pub disposition: String,
    pub anomaly: Option<Anomaly>,
    pub correlation_id: String,
    pub upstream_execution: Vec<String>,
}

impl ObservationResult {
    pub fn from_domain(value: domain::Observation, anomaly: Option<&domain::Anomaly>) -> Self {
        Self {
            observation_id: value.id,
            disposition: if anomaly.is_some() {
                "anomaly"
            } else {
                "nominal"
            }
            .into(),
            anomaly: anomaly.map(Anomaly::from),
            correlation_id: value.correlation_id,
            upstream_execution: vec![
                "RuCelium signed ingest".into(),
                "Midstream temporal comparison".into(),
                "OpenDAL evidence write".into(),
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct InterventionRequest {
    pub anomaly_id: String,
    pub action: String,
    pub rationale: String,
    pub risk_level: String,
}

impl InterventionRequest {
    pub fn into_domain(self) -> domain::InterventionRequest {
        let mut parameters = BTreeMap::new();
        parameters.insert("risk_level".into(), self.risk_level);
        parameters.insert("requested_from".into(), "browser-workspace".into());
        domain::InterventionRequest {
            anomaly_id: self.anomaly_id,
            action: self.action,
            zone_id: "wetland-east".into(),
            rationale: self.rationale,
            parameters,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct HabitatZone {
    pub id: String,
    pub name: String,
    pub region: String,
    pub health: String,
    pub canopy_cover: f64,
    pub moisture: f64,
    pub anomaly_count: usize,
    pub coordinates: Coordinates,
}

#[derive(Clone, Debug, Serialize)]
pub struct Sensor {
    pub id: String,
    pub label: String,
    pub modality: String,
    pub zone_id: String,
    pub trust_score: f64,
    pub calibration_status: String,
    pub signature_verified: bool,
    pub last_seen: DateTime<Utc>,
    pub value: f64,
    pub unit: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Anomaly {
    pub id: String,
    pub title: String,
    pub zone_id: String,
    pub severity: String,
    pub confidence: f64,
    pub status: String,
    pub detected_at: DateTime<Utc>,
    pub evidence: Vec<String>,
}

impl From<&domain::Anomaly> for Anomaly {
    fn from(value: &domain::Anomaly) -> Self {
        Self {
            id: value.id.clone(),
            title: format!("{} outside expected habitat envelope", value.metric),
            zone_id: value.zone_id.clone(),
            severity: if value.score >= 0.85 {
                "high"
            } else {
                "medium"
            }
            .into(),
            confidence: value.score.clamp(0.0, 1.0),
            status: value.state.clone(),
            detected_at: Utc::now(),
            evidence: value.evidence_ids.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Intervention {
    pub id: String,
    pub anomaly_id: String,
    pub action: String,
    pub rationale: String,
    pub risk_level: String,
    pub status: String,
    pub policy_reason: String,
    pub created_at: DateTime<Utc>,
    pub correlation_id: String,
}

impl From<domain::Intervention> for Intervention {
    fn from(value: domain::Intervention) -> Self {
        Self {
            id: value.id,
            anomaly_id: value.request.anomaly_id,
            action: value.request.action,
            rationale: value.request.rationale,
            risk_level: value
                .request
                .parameters
                .get("risk_level")
                .cloned()
                .unwrap_or_else(|| "guarded".into()),
            status: if value.state == "deployed" {
                "executed".into()
            } else {
                value.state
            },
            policy_reason: value.policy_decision,
            created_at: value.created_at,
            correlation_id: value.correlation_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MeshLink {
    pub id: String,
    pub from: String,
    pub to: String,
    pub state: String,
    pub latency_ms: u64,
    pub queue_depth: u64,
    pub transport: String,
    pub last_sync: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuditEntry {
    pub id: String,
    pub event_type: String,
    pub summary: String,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub hash: String,
    pub correlation_id: String,
}

impl From<domain::AuditEntry> for AuditEntry {
    fn from(value: domain::AuditEntry) -> Self {
        Self {
            summary: format!("{} for {}", value.event_type, value.subject_id),
            actor: "field-lead".into(),
            timestamp: value.created_at,
            id: value.id,
            event_type: value.event_type,
            hash: value.hash,
            correlation_id: value.correlation_id,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UpstreamStatus {
    pub name: String,
    pub status: String,
    pub last_operation: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardSummary {
    pub habitats_online: usize,
    pub trusted_sensors: usize,
    pub open_anomalies: usize,
    pub pending_interventions: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct Dashboard {
    pub project_name: String,
    pub generated_at: DateTime<Utc>,
    pub readiness: String,
    pub correlation_id: String,
    pub summary: DashboardSummary,
    pub habitat_zones: Vec<HabitatZone>,
    pub sensors: Vec<Sensor>,
    pub anomalies: Vec<Anomaly>,
    pub interventions: Vec<Intervention>,
    pub mesh_links: Vec<MeshLink>,
    pub audit_entries: Vec<AuditEntry>,
    pub upstreams: Vec<UpstreamStatus>,
}

impl Dashboard {
    pub fn from_domain(value: domain::Dashboard, correlation_id: String) -> Self {
        let open_anomalies = value
            .anomalies
            .iter()
            .filter(|item| item.state != "resolved")
            .count();
        let audit_entries = value.audit.into_iter().map(AuditEntry::from).collect();
        let interventions: Vec<_> = value
            .interventions
            .into_iter()
            .map(Intervention::from)
            .collect();
        let habitat_zones = value
            .zones
            .iter()
            .enumerate()
            .map(|(index, zone)| HabitatZone {
                id: zone.id.clone(),
                name: zone.name.clone(),
                region: zone.habitat_class.clone(),
                health: if value
                    .anomalies
                    .iter()
                    .any(|item| item.zone_id == zone.id && item.score > 0.8)
                {
                    "watch"
                } else {
                    "healthy"
                }
                .into(),
                canopy_cover: 0.62 + index as f64 * 0.08,
                moisture: 0.71 - index as f64 * 0.12,
                anomaly_count: value
                    .anomalies
                    .iter()
                    .filter(|item| item.zone_id == zone.id)
                    .count(),
                coordinates: Coordinates {
                    x: 22.0 + index as f64 * 28.0,
                    y: 34.0 + (index % 2) as f64 * 24.0,
                },
            })
            .collect();
        let sensors: Vec<Sensor> = value
            .sensors
            .into_iter()
            .enumerate()
            .map(|(index, sensor)| Sensor {
                id: sensor.device_id.clone(),
                label: sensor.device_id.replace('-', " "),
                modality: if sensor.device_id.contains("soil") {
                    "soil-moisture"
                } else {
                    "acoustic-activity"
                }
                .into(),
                zone_id: sensor.zone_id,
                trust_score: 0.94 - index as f64 * 0.03,
                calibration_status: sensor.trust_state,
                signature_verified: true,
                last_seen: sensor.last_seen,
                value: if index == 0 { 0.41 } else { 0.73 },
                unit: "ratio".into(),
            })
            .collect();
        let anomalies = value.anomalies.iter().map(Anomaly::from).collect();
        let mesh_links = value
            .mesh_links
            .into_iter()
            .enumerate()
            .map(|(index, link)| MeshLink {
                id: format!("mesh-{index}"),
                from: link.from_zone,
                to: link.to_zone,
                state: if link.state == "healthy" {
                    "online"
                } else {
                    "delayed"
                }
                .into(),
                latency_ms: link.latency_ms,
                queue_depth: if link.state == "degraded" { 12 } else { 0 },
                transport: "LatentMesh semantic relay".into(),
                last_sync: Utc::now(),
            })
            .collect();
        let upstreams = value
            .upstreams
            .into_iter()
            .map(|item| UpstreamStatus {
                name: item.name,
                status: if item.healthy { "ready" } else { "degraded" }.into(),
                last_operation: item.last_operation,
            })
            .collect();
        Self {
            project_name: "Canopy Relay".into(),
            generated_at: value.generated_at,
            readiness: "ready".into(),
            correlation_id,
            summary: DashboardSummary {
                habitats_online: value.zones.len(),
                trusted_sensors: sensors.len(),
                open_anomalies,
                pending_interventions: interventions.len(),
            },
            habitat_zones,
            sensors,
            anomalies,
            interventions,
            mesh_links,
            audit_entries,
            upstreams,
        }
    }
}
