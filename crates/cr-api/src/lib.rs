//! Versioned Canopy Relay HTTP handlers matching the checked TypeScript client.
mod view;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use cr_application::CanopyRelay;
use cr_contracts::{Envelope, Problem};
use cr_errors::RelayError;
use cr_types::AuditEntry;
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiState {
    pub relay: Arc<CanopyRelay>,
}

pub type ApiResult<T> = std::result::Result<Json<Envelope<T>>, ApiError>;

pub struct ApiError {
    error: RelayError,
    correlation_id: String,
}

impl ApiError {
    fn new(error: RelayError, correlation_id: impl Into<String>) -> Self {
        Self {
            error,
            correlation_id: correlation_id.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.error {
            RelayError::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            RelayError::Denied(_) => StatusCode::FORBIDDEN,
            RelayError::NotFound(_) => StatusCode::NOT_FOUND,
            RelayError::Conflict(_) => StatusCode::CONFLICT,
            RelayError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            RelayError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            _ => StatusCode::BAD_GATEWAY,
        };
        (
            status,
            Json(Problem::from_error(&self.error, self.correlation_id)),
        )
            .into_response()
    }
}

fn authenticate(
    headers: &HeaderMap,
) -> std::result::Result<cr_middleware::RequestContext, ApiError> {
    let bearer = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());
    let correlation = headers
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok());
    cr_middleware::authenticate(bearer, correlation)
        .map_err(|error| ApiError::new(error, correlation.unwrap_or("unassigned")))
}

pub async fn dashboard(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> ApiResult<view::Dashboard> {
    let context = authenticate(&headers)?;
    let value = state
        .relay
        .dashboard()
        .map_err(|error| ApiError::new(error, &context.correlation_id))?;
    Ok(Json(
        Envelope::v1(
            context.correlation_id.clone(),
            view::Dashboard::from_domain(value, context.correlation_id),
        )
        .map_err(|error| ApiError::new(error, "dashboard"))?,
    ))
}

pub async fn create_observation(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<view::ObservationRequest>,
) -> ApiResult<view::ObservationResult> {
    let context = authenticate(&headers)?;
    let receipt = headers
        .get("x-device-trust-receipt")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            ApiError::new(
                RelayError::Unauthenticated("device trust receipt required".into()),
                &context.correlation_id,
            )
        })?;
    let value = state
        .relay
        .observe(input.into_domain(), receipt, &context.correlation_id)
        .await
        .map_err(|error| ApiError::new(error, &context.correlation_id))?;
    Ok(Json(
        Envelope::v1(
            context.correlation_id,
            view::ObservationResult::from_domain(
                value,
                state
                    .relay
                    .dashboard()
                    .ok()
                    .and_then(|item| item.anomalies.into_iter().next())
                    .as_ref(),
            ),
        )
        .map_err(|error| ApiError::new(error, "observation"))?,
    ))
}

pub async fn create_intervention(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Json(input): Json<view::InterventionRequest>,
) -> ApiResult<view::Intervention> {
    let context = authenticate(&headers)?;
    cr_middleware::require_role(&context, "field-lead")
        .map_err(|error| ApiError::new(error, &context.correlation_id))?;
    let value = state
        .relay
        .intervene("field-lead", input.into_domain(), &context.correlation_id)
        .await
        .map_err(|error| ApiError::new(error, &context.correlation_id))?;
    Ok(Json(
        Envelope::v1(context.correlation_id, view::Intervention::from(value))
            .map_err(|error| ApiError::new(error, "intervention"))?,
    ))
}

pub async fn list_interventions(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> ApiResult<Vec<view::Intervention>> {
    let context = authenticate(&headers)?;
    Ok(Json(
        Envelope::v1(
            context.correlation_id,
            state
                .relay
                .interventions()
                .into_iter()
                .map(view::Intervention::from)
                .collect(),
        )
        .map_err(|error| ApiError::new(error, "interventions"))?,
    ))
}

pub async fn list_audit(
    State(state): State<ApiState>,
    headers: HeaderMap,
) -> ApiResult<Vec<AuditEntry>> {
    let context = authenticate(&headers)?;
    Ok(Json(
        Envelope::v1(context.correlation_id, state.relay.audit_entries())
            .map_err(|error| ApiError::new(error, "audit"))?,
    ))
}

pub async fn get_audit(
    State(state): State<ApiState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<AuditEntry> {
    let context = authenticate(&headers)?;
    let entry = state
        .relay
        .audit_entries()
        .into_iter()
        .find(|entry| entry.id == id)
        .ok_or_else(|| ApiError::new(RelayError::NotFound(id), &context.correlation_id))?;
    Ok(Json(
        Envelope::v1(context.correlation_id, entry)
            .map_err(|error| ApiError::new(error, "audit-entry"))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response_mapping_distinguishes_denial_and_validation() {
        let denial = ApiError::new(RelayError::Denied("x".into()), "corr").into_response();
        let invalid = ApiError::new(RelayError::Validation("x".into()), "corr").into_response();
        assert_eq!(denial.status(), StatusCode::FORBIDDEN);
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
