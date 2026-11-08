//! Stable versioned API contracts shared by handlers and contract tests.
use cr_errors::{RelayError, Result};
use cr_types::{Dashboard, Intervention, InterventionRequest, Observation, ObservationInput};
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "v1";
pub const API_PREFIX: &str = "/api/v1";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Envelope<T> {
    pub schema_version: String,
    pub correlation_id: String,
    pub data: T,
}

impl<T> Envelope<T> {
    pub fn v1(correlation_id: impl Into<String>, data: T) -> Result<Self> {
        let correlation_id = correlation_id.into();
        if correlation_id.trim().is_empty() || correlation_id.len() > 128 {
            return Err(RelayError::Validation("invalid correlation id".into()));
        }
        Ok(Self {
            schema_version: API_VERSION.into(),
            correlation_id,
            data,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Problem {
    pub schema_version: String,
    pub code: String,
    pub message: String,
    pub correlation_id: String,
    pub retryable: bool,
}

impl Problem {
    pub fn from_error(error: &RelayError, correlation_id: impl Into<String>) -> Self {
        Self {
            schema_version: API_VERSION.into(),
            code: error.code().into(),
            message: error.to_string(),
            correlation_id: correlation_id.into(),
            retryable: matches!(error, RelayError::Upstream { .. } | RelayError::Timeout(_)),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "payload", rename_all = "kebab-case")]
pub enum ApiResource {
    Dashboard(Dashboard),
    Observation(Observation),
    Intervention(Intervention),
    InterventionList(Vec<Intervention>),
}

pub fn validate_observation_request(value: &ObservationInput) -> Result<()> {
    if value.device_id.trim().is_empty() || value.zone_id.trim().is_empty() {
        return Err(RelayError::Validation(
            "device and zone are required".into(),
        ));
    }
    if value.species.len() > 120 || value.metric.len() > 80 || value.unit.len() > 32 {
        return Err(RelayError::Validation(
            "observation field exceeds contract".into(),
        ));
    }
    if !value.location.valid() || !value.value.is_finite() {
        return Err(RelayError::Validation(
            "invalid coordinate or measurement".into(),
        ));
    }
    Ok(())
}

pub fn validate_intervention_request(value: &InterventionRequest) -> Result<()> {
    if value.anomaly_id.trim().is_empty()
        || value.zone_id.trim().is_empty()
        || value.action.trim().is_empty()
    {
        return Err(RelayError::Validation(
            "anomaly, zone and action are required".into(),
        ));
    }
    if value.rationale.len() < 12 || value.rationale.len() > 2_000 {
        return Err(RelayError::Validation(
            "rationale must contain 12-2000 characters".into(),
        ));
    }
    if value.parameters.len() > 32
        || value
            .parameters
            .iter()
            .any(|(key, item)| key.len() > 64 || item.len() > 512)
    {
        return Err(RelayError::Validation(
            "intervention parameters exceed contract".into(),
        ));
    }
    Ok(())
}

pub fn canonical_paths() -> [&'static str; 6] {
    [
        "/api/v1/dashboard",
        "/api/v1/observations",
        "/api/v1/interventions",
        "/api/v1/interventions",
        "/api/v1/audit",
        "/api/v1/audit/{id}",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn restricted_short_rationale_is_rejected_by_contract() {
        let request = InterventionRequest {
            anomaly_id: "anomaly-1".into(),
            action: "deploy-mesh-relay".into(),
            zone_id: "wetland-east".into(),
            rationale: "short".into(),
            parameters: BTreeMap::new(),
        };
        assert!(validate_intervention_request(&request).is_err());
    }

    #[test]
    fn error_contract_marks_dependency_failures_retryable() {
        let error = RelayError::upstream("worldgraph", "not ready");
        let problem = Problem::from_error(&error, "corr-1");
        assert!(problem.retryable);
        assert_eq!(problem.code, "upstream");
    }
}
