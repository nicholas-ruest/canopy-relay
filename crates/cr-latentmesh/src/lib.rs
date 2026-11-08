//! Direct LatentMesh constrained-radio fragmentation boundary.
use cr_errors::{RelayError, Result};
use cr_types::{Anomaly, Forecast, MeshLink, Observation, UpstreamStatus};
use latentmesh_air_core::{
    fragment_message, FragmentMeta, FrameFlags, Reassembler, ReassemblerConfig, SemanticClass,
    WireProfile,
};
use serde::{Deserialize, Serialize};
const REVISION: &str = "df9270799a1391925c1840a1845a6d169aaa2d93";
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
        let route = route_for(r);
        let bytes = serde_json::to_vec(&route).map_err(|e| RelayError::Integrity(e.to_string()))?;
        let meta = FragmentMeta {
            profile: WireProfile::Ble,
            flags: FrameFlags::NONE,
            stream_id: 7,
            sequence: 1,
            class: SemanticClass::StateDelta,
            priority: 8,
            state_tag: 0x4341,
        };
        let frames = fragment_message(meta, &bytes, 64)
            .map_err(|e| RelayError::upstream("latentmesh", e.to_string()))?;
        let frame_count = frames.len();
        let mut receiver = Reassembler::new(ReassemblerConfig::default())
            .map_err(|e| RelayError::upstream("latentmesh", e.to_string()))?;
        let mut completed = None;
        for frame in frames.into_iter().rev() {
            if let Some(message) = receiver
                .push(frame)
                .map_err(|e| RelayError::upstream("latentmesh", e.to_string()))?
            {
                completed = Some(message.bytes)
            }
        }
        let decoded: Vec<String> = serde_json::from_slice(
            &completed
                .ok_or_else(|| RelayError::Integrity("latentmesh message incomplete".into()))?,
        )
        .map_err(|e| RelayError::Integrity(e.to_string()))?;
        Ok(Response {
            upstream: "latentmesh".into(),
            revision: REVISION.into(),
            operation: "fragment_and_route_evidence".into(),
            score: (1.0 / (frame_count as f64)).clamp(0.2, 1.0),
            route: decoded,
            forecasts: vec![],
            evidence: serde_json::json!({"fragments":frame_count,"reassembled":true}),
        })
    }
    pub fn status(&self) -> UpstreamStatus {
        UpstreamStatus {
            name: "latentmesh".into(),
            revision: REVISION.into(),
            healthy: true,
            last_operation: "fragment_and_route_evidence".into(),
        }
    }
}
fn route_for(r: &Request) -> Vec<String> {
    let mut route = vec![r.zone_id.clone()];
    for link in &r.links {
        if link.state == "online" && (link.from_zone == *route.last().unwrap_or(&r.zone_id)) {
            route.push(link.to_zone.clone())
        }
    }
    if route.len() == 1 {
        route.push("field-gateway".into())
    }
    route
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn fragments_and_reassembles_route() {
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
        assert_eq!(r.route.last().unwrap(), "field-gateway")
    }
    #[test]
    fn upstream_rejects_invalid_mtu() {
        let meta = FragmentMeta {
            profile: WireProfile::Ble,
            flags: FrameFlags::NONE,
            stream_id: 1,
            sequence: 1,
            class: SemanticClass::StateDelta,
            priority: 1,
            state_tag: 1,
        };
        assert!(fragment_message(meta, b"x", 1).is_err())
    }
}
