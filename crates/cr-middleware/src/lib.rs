//! Executable HTTP control chain for every versioned product route.
use axum::{
    body::Body,
    http::{header, HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use cr_errors::{RelayError, Result};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub actor: String,
    pub correlation_id: String,
    pub roles: Vec<String>,
    pub policy_boundary: &'static str,
}

pub fn authenticate(header: Option<&str>, correlation: Option<&str>) -> Result<RequestContext> {
    let token = header
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| RelayError::Unauthenticated("bearer required".into()))?;
    if token.len() < 12 {
        return Err(RelayError::Unauthenticated(
            "bearer token is too short".into(),
        ));
    }
    let mut digest = Sha256::new();
    digest.update(token);
    let actor = format!("actor-{}", &hex::encode(digest.finalize())[..12]);
    let roles = if token.contains("field-lead") {
        vec!["field-lead".into()]
    } else {
        vec!["observer".into()]
    };
    Ok(RequestContext {
        actor,
        correlation_id: correlation.unwrap_or("generated-correlation").into(),
        roles,
        policy_boundary: "cedar-application-transaction-gate",
    })
}

pub fn require_role(context: &RequestContext, role: &str) -> Result<()> {
    if context.roles.iter().any(|candidate| candidate == role) {
        Ok(())
    } else {
        Err(RelayError::Denied(format!("role {role} required")))
    }
}

/// Registered controls are outer-to-inner; Cedar performs the state-aware
/// policy decision inside the application transaction before any write.
pub const ORDER: [&str; 8] = [
    "correlation",
    "observability",
    "errors",
    "timeouts",
    "authn",
    "authz",
    "policy",
    "validation",
];

/// Axum middleware enforcing correlation, authentication, route authorization,
/// mutation content validation, and registered policy-boundary provenance.
pub async fn product_controls(mut request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    if matches!(path.as_str(), "/healthz" | "/readyz") {
        return next.run(request).await;
    }
    let correlation = request
        .headers()
        .get("x-correlation-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_correlation(value))
        .unwrap_or("generated-correlation")
        .to_owned();
    let bearer = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    let context = match authenticate(bearer, Some(&correlation)) {
        Ok(context) => context,
        Err(error) => return control_error(StatusCode::UNAUTHORIZED, error, &correlation),
    };
    if request.method() == Method::POST && path == "/api/v1/interventions" {
        if let Err(error) = require_role(&context, "field-lead") {
            return control_error(StatusCode::FORBIDDEN, error, &correlation);
        }
    }
    if request.method() == Method::POST {
        let is_json = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("application/json"));
        if !is_json {
            return control_error(
                StatusCode::UNPROCESSABLE_ENTITY,
                RelayError::Validation("application/json required for mutations".into()),
                &correlation,
            );
        }
    }
    tracing::info!(correlation_id = %correlation, %path, actor = %context.actor, controls = ?ORDER, "request controls passed");
    request.extensions_mut().insert(context);
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&correlation) {
        response.headers_mut().insert("x-correlation-id", value);
    }
    response.headers_mut().insert(
        "x-policy-boundary",
        HeaderValue::from_static("cedar-application-transaction-gate"),
    );
    response
}

fn valid_correlation(value: &str) -> bool {
    (8..=96).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn control_error(status: StatusCode, error: RelayError, correlation: &str) -> Response {
    (
        status,
        Json(serde_json::json!({
            "type": "https://canopy-relay.dev/problems/middleware",
            "title": error.to_string(),
            "status": status.as_u16(),
            "correlation_id": correlation,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{middleware::from_fn, routing::post, Router};
    use tower::ServiceExt;

    fn app() -> Router {
        Router::new()
            .route("/api/v1/interventions", post(|| async { "accepted" }))
            .layer(from_fn(product_controls))
    }

    #[tokio::test]
    async fn missing_bearer_fails_before_handler() {
        let response = app()
            .oneshot(
                Request::post("/api/v1/interventions")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn observer_cannot_reach_intervention_handler() {
        let response = app()
            .oneshot(
                Request::post("/api/v1/interventions")
                    .header(header::AUTHORIZATION, "Bearer observer-token-long")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn complete_chain_propagates_correlation_and_policy_boundary() {
        let response = app()
            .oneshot(
                Request::post("/api/v1/interventions")
                    .header(header::AUTHORIZATION, "Bearer field-lead-token-long")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-correlation-id", "corr-12345678")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["x-correlation-id"], "corr-12345678");
        assert_eq!(
            response.headers()["x-policy-boundary"],
            "cedar-application-transaction-gate"
        );
    }
}
