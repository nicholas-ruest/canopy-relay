//! Cedar-backed authorization for habitat interventions.
use cedar_policy::{Authorizer, Context, Decision, Entities, EntityUid, PolicySet, Request};
use cr_errors::{RelayError, Result};
use cr_types::InterventionRequest;

/// Compiled policy set plus the Cedar evaluator used on every intervention.
pub struct CedarAuthorizer {
    policies: PolicySet,
    authorizer: Authorizer,
}

impl CedarAuthorizer {
    /// Parse policy text at service startup, failing closed on invalid policy.
    pub fn parse(policy: &str) -> Result<Self> {
        Ok(Self {
            policies: policy
                .parse::<PolicySet>()
                .map_err(|error| RelayError::Validation(error.to_string()))?,
            authorizer: Authorizer::new(),
        })
    }

    /// Evaluate the concrete operator, action, and habitat zone through Cedar.
    pub fn authorize(&self, actor: &str, request: &InterventionRequest) -> Result<String> {
        let principal = entity_uid("Operator", actor)?;
        let action = entity_uid("Action", &request.action)?;
        let resource = entity_uid("HabitatZone", &request.zone_id)?;
        let cedar_request = Request::new(principal, action, resource, Context::empty(), None)
            .map_err(|error| RelayError::Validation(error.to_string()))?;
        let response =
            self.authorizer
                .is_authorized(&cedar_request, &self.policies, &Entities::empty());
        match response.decision() {
            Decision::Allow => Ok("allow".into()),
            Decision::Deny => Err(RelayError::Denied(format!(
                "cedar denied {} for {actor}",
                request.action
            ))),
        }
    }
}

fn entity_uid(entity_type: &str, id: &str) -> Result<EntityUid> {
    if id.is_empty() || id.contains(['"', '\\', '\n', '\r']) {
        return Err(RelayError::Validation(format!(
            "unsafe Cedar {entity_type} identifier"
        )));
    }
    format!(r#"{entity_type}::"{id}""#)
        .parse()
        .map_err(|error| RelayError::Validation(format!("invalid Cedar entity: {error}")))
}

pub const DEFAULT_POLICY: &str = r#"
permit (
    principal == Operator::"field-lead",
    action == Action::"deploy-mesh-relay",
    resource
);
permit (
    principal == Operator::"field-lead",
    action == Action::"schedule-calibration",
    resource
);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn request(action: &str) -> InterventionRequest {
        InterventionRequest {
            anomaly_id: "anomaly-1".into(),
            action: action.into(),
            zone_id: "cedar-ridge".into(),
            rationale: "restore sensor coverage".into(),
            parameters: BTreeMap::new(),
        }
    }

    #[test]
    fn cedar_permits_safe_field_lead_action() {
        let authorizer = CedarAuthorizer::parse(DEFAULT_POLICY).unwrap();
        assert_eq!(
            authorizer
                .authorize("field-lead", &request("deploy-mesh-relay"))
                .unwrap(),
            "allow"
        );
    }

    #[test]
    fn cedar_denies_restricted_action_and_wrong_actor() {
        let authorizer = CedarAuthorizer::parse(DEFAULT_POLICY).unwrap();
        assert!(authorizer
            .authorize("field-lead", &request("controlled-burn"))
            .is_err());
        assert!(authorizer
            .authorize("observer", &request("deploy-mesh-relay"))
            .is_err());
    }
}
