//! Deterministic synthetic ecological seed entering through real domain types.
use chrono::{TimeZone, Utc};
use cr_errors::{RelayError, Result};
use cr_types::{Anomaly, GeoPoint, HabitatZone, MeshLink, SensorSummary};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct SeedWorld {
    pub zones: Vec<HabitatZone>,
    pub sensors: Vec<SensorSummary>,
    pub anomalies: Vec<Anomaly>,
    pub links: Vec<MeshLink>,
}

impl SeedWorld {
    pub fn validate(&self) -> Result<()> {
        let zone_ids: BTreeSet<_> = self.zones.iter().map(|zone| zone.id.as_str()).collect();
        if zone_ids.is_empty() || zone_ids.len() != self.zones.len() {
            return Err(RelayError::Integrity(
                "seed requires unique habitat zones".into(),
            ));
        }
        if self.zones.iter().any(|zone| {
            !zone.centroid.valid()
                || zone
                    .neighbors
                    .iter()
                    .any(|id| !zone_ids.contains(id.as_str()))
        }) || self
            .sensors
            .iter()
            .any(|sensor| !zone_ids.contains(sensor.zone_id.as_str()))
            || self
                .anomalies
                .iter()
                .any(|anomaly| !zone_ids.contains(anomaly.zone_id.as_str()))
            || self.links.iter().any(|link| {
                !zone_ids.contains(link.from_zone.as_str())
                    || !zone_ids.contains(link.to_zone.as_str())
            })
        {
            return Err(RelayError::Integrity(
                "seed contains invalid geometry or unknown zone reference".into(),
            ));
        }
        Ok(())
    }
}

pub fn toronto_ravine() -> SeedWorld {
    let observed = Utc
        .with_ymd_and_hms(2026, 11, 7, 12, 0, 0)
        .single()
        .expect("fixed seed date");
    SeedWorld {
        zones: vec![
            HabitatZone {
                id: "wetland-east".into(),
                name: "East Wetland".into(),
                centroid: GeoPoint {
                    lat: 43.642,
                    lon: -79.374,
                },
                habitat_class: "urban-wetland".into(),
                sensitivity: "protected".into(),
                neighbors: vec!["canopy-north".into()],
            },
            HabitatZone {
                id: "canopy-north".into(),
                name: "North Canopy".into(),
                centroid: GeoPoint {
                    lat: 43.647,
                    lon: -79.371,
                },
                habitat_class: "deciduous-canopy".into(),
                sensitivity: "managed".into(),
                neighbors: vec!["wetland-east".into(), "meadow-west".into()],
            },
            HabitatZone {
                id: "meadow-west".into(),
                name: "West Meadow".into(),
                centroid: GeoPoint {
                    lat: 43.644,
                    lon: -79.382,
                },
                habitat_class: "pollinator-meadow".into(),
                sensitivity: "managed".into(),
                neighbors: vec!["canopy-north".into()],
            },
        ],
        sensors: vec![
            SensorSummary {
                device_id: "soil-7".into(),
                zone_id: "wetland-east".into(),
                trust_state: "verified".into(),
                last_seen: observed,
                calibration_version: "soil-v3".into(),
            },
            SensorSummary {
                device_id: "acoustic-2".into(),
                zone_id: "canopy-north".into(),
                trust_state: "verified".into(),
                last_seen: observed,
                calibration_version: "audio-v2".into(),
            },
        ],
        anomalies: vec![Anomaly {
            id: "anomaly-mesh-loss".into(),
            zone_id: "wetland-east".into(),
            metric: "relay-connectivity".into(),
            score: 0.91,
            observed: 0.23,
            expected: 0.87,
            evidence_ids: vec!["soil-7-window-22".into(), "relay-4-heartbeat".into()],
            state: "open".into(),
        }],
        links: vec![
            MeshLink {
                from_zone: "wetland-east".into(),
                to_zone: "canopy-north".into(),
                capacity: 0.31,
                ecological_cost: 0.08,
                latency_ms: 410,
                state: "degraded".into(),
            },
            MeshLink {
                from_zone: "canopy-north".into(),
                to_zone: "meadow-west".into(),
                capacity: 0.83,
                ecological_cost: 0.05,
                latency_ms: 74,
                state: "healthy".into(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seed_is_connected_and_has_recovery_target() {
        let seed = toronto_ravine();
        seed.validate().unwrap();
        assert_eq!(seed.zones.len(), 3);
        assert!(seed.links.iter().any(|link| link.state == "degraded"));
        assert!(seed
            .anomalies
            .iter()
            .any(|item| item.id == "anomaly-mesh-loss"));
    }

    #[test]
    fn seed_rejects_unknown_zone_references() {
        let mut seed = toronto_ravine();
        seed.sensors[0].zone_id = "missing-zone".into();
        assert!(seed.validate().is_err());
    }
}
