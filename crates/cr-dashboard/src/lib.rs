//! Domain projection for the operational digital twin.
use chrono::Utc;
use cr_errors::{RelayError, Result};
use cr_types::{
    Anomaly, AuditEntry, Dashboard, HabitatZone, Intervention, MeshLink, SensorSummary,
    UpstreamStatus,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub struct DashboardBuilder {
    zones: Vec<HabitatZone>,
    sensors: Vec<SensorSummary>,
    anomalies: Vec<Anomaly>,
    interventions: Vec<Intervention>,
    links: Vec<MeshLink>,
    audit: Vec<AuditEntry>,
    upstreams: Vec<UpstreamStatus>,
}

impl DashboardBuilder {
    pub fn topology(mut self, zones: Vec<HabitatZone>, links: Vec<MeshLink>) -> Self {
        self.zones = zones;
        self.links = links;
        self
    }
    pub fn sensor_state(mut self, sensors: Vec<SensorSummary>) -> Self {
        self.sensors = sensors;
        self
    }
    pub fn operational_state(
        mut self,
        anomalies: Vec<Anomaly>,
        interventions: Vec<Intervention>,
    ) -> Self {
        self.anomalies = anomalies;
        self.interventions = interventions;
        self
    }
    pub fn evidence(mut self, audit: Vec<AuditEntry>) -> Self {
        self.audit = audit;
        self
    }
    pub fn dependencies(mut self, upstreams: Vec<UpstreamStatus>) -> Self {
        self.upstreams = upstreams;
        self
    }
    pub fn build(self) -> Result<Dashboard> {
        let zones: BTreeSet<_> = self.zones.iter().map(|zone| zone.id.as_str()).collect();
        if zones.len() != self.zones.len() || zones.is_empty() {
            return Err(RelayError::Integrity(
                "dashboard needs unique habitat zones".into(),
            ));
        }
        if self
            .sensors
            .iter()
            .any(|sensor| !zones.contains(sensor.zone_id.as_str()))
            || self
                .anomalies
                .iter()
                .any(|anomaly| !zones.contains(anomaly.zone_id.as_str()))
        {
            return Err(RelayError::Integrity(
                "dashboard references an unknown habitat zone".into(),
            ));
        }
        let audit_by_subject: BTreeMap<_, _> = self
            .audit
            .iter()
            .map(|entry| (entry.subject_id.as_str(), entry.sequence))
            .collect();
        if self
            .interventions
            .iter()
            .any(|item| !audit_by_subject.contains_key(item.id.as_str()))
        {
            return Err(RelayError::Integrity(
                "intervention lacks persisted audit evidence".into(),
            ));
        }
        Ok(Dashboard {
            zones: self.zones,
            sensors: self.sensors,
            anomalies: self.anomalies,
            interventions: self.interventions,
            mesh_links: self.links,
            audit: self.audit,
            upstreams: self.upstreams,
            generated_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_dashboard_is_not_a_product_projection() {
        assert!(DashboardBuilder::default().build().is_err());
    }
}
