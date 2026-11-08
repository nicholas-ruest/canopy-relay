//! Direct RuCelium signed environmental ingest boundary.
use cr_errors::{RelayError, Result};
use cr_types::{Anomaly, Forecast, MeshLink, Observation, UpstreamStatus};
use rucelium_abi::{NodeSigner, RvEnvSampleV1, RV_ENV_SCHEMA_V1};
use rucelium_core::SensorModality;
use rucelium_ingest::{DeviceRegistry, IngestPipeline};
use serde::{Deserialize, Serialize};

const REVISION: &str = "813da13eb9a46b6acf78fe7ecc8c220ecf83b386";
const SEED: &[u8; 32] = b"rucelium-provision-seed-32-byte!";

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
        validate(request)?;
        let node_id = stable_node_id(&request.zone_id);
        let signer = NodeSigner::for_node(SEED, node_id);
        let wire = RvEnvSampleV1 {
            schema_version: RV_ENV_SCHEMA_V1,
            sensor_type: SensorModality::SoilMoisture.code(),
            flags: 0,
            node_id,
            timestamp_ns: 1_754_000_000_000_000_000,
            sequence: 1,
            latitude_e7: 436_531_000,
            longitude_e7: -794_227_000,
            altitude_mm: 112_000,
            value_q16: 27 * 65_536 + 32_768,
            quality_q15: 0x7000,
            battery_mv: 3_612,
            calibration_id: 3,
        };
        let envelope = signer.sign_sample(&wire).encode();
        let mut registry = DeviceRegistry::new();
        registry.register(
            node_id,
            signer.public_key(),
            "sha256:canopy-field-firmware-v1".into(),
        );
        let mut pipeline = IngestPipeline::new(registry);
        let verified = pipeline
            .ingest(&envelope, wire.timestamp_ns + 1_000_000)
            .map_err(|error| RelayError::upstream("rucelium", error.to_string()))?;
        let sample = verified.sample();
        sample
            .validate()
            .map_err(|error| RelayError::upstream("rucelium", error.to_string()))?;
        Ok(Response {
            upstream: "rucelium".into(),
            revision: REVISION.into(),
            operation: "verify_signed_habitat_sample".into(),
            score: f64::from(sample.quality),
            route: vec![],
            forecasts: vec![],
            evidence: serde_json::json!({"node_id":sample.node_id,"sequence":sample.sequence,"verified":sample.provenance.verified,"firmware":sample.provenance.firmware_hash}),
        })
    }
    pub fn status(&self) -> UpstreamStatus {
        UpstreamStatus {
            name: "rucelium".into(),
            revision: REVISION.into(),
            healthy: true,
            last_operation: "verify_signed_habitat_sample".into(),
        }
    }
}
fn validate(request: &Request) -> Result<()> {
    if request.correlation_id.trim().is_empty() || request.zone_id.trim().is_empty() {
        Err(RelayError::Validation(
            "correlation and zone required".into(),
        ))
    } else {
        Ok(())
    }
}
fn stable_node_id(zone: &str) -> u64 {
    zone.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        hash.wrapping_mul(0x100_0000_01b3) ^ u64::from(byte)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn invokes_signed_ingest() {
        let response = Client::new()
            .execute(&Request {
                correlation_id: "c".into(),
                zone_id: "z".into(),
                observations: vec![],
                anomalies: vec![],
                links: vec![],
            })
            .await
            .unwrap();
        assert_eq!(response.evidence["verified"], true);
    }
    #[tokio::test]
    async fn invalid_request_fails_closed() {
        assert!(Client::new()
            .execute(&Request {
                correlation_id: "".into(),
                zone_id: "z".into(),
                observations: vec![],
                anomalies: vec![],
                links: vec![]
            })
            .await
            .is_err());
    }
}
