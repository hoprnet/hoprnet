use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;

use crate::{ApiError, ApiErrorStatus, InternalState};

#[cfg(all(feature = "prometheus", not(test)))]
fn collect_hopr_metrics() -> Result<String, ApiErrorStatus> {
    hopr_metrics::metrics::gather_all_metrics()
        .map_err(|_| ApiErrorStatus::UnknownFailure("Failed to gather metrics".into()))
}

#[cfg(any(not(feature = "prometheus"), test))]
fn collect_hopr_metrics() -> Result<String, ApiErrorStatus> {
    Err(ApiErrorStatus::UnknownFailure("BUILT WITHOUT METRICS SUPPORT".into()))
}

/// Retrieve Prometheus metrics from the running node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("/metrics"),
        description = "Retrieve Prometheus metrics from the running node",
        responses(
            (status = 200, description = "Fetched node metrics", body = String),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Metrics"
    )]
pub(super) async fn metrics() -> impl IntoResponse {
    match collect_hopr_metrics() {
        Ok(metrics) => (StatusCode::OK, metrics).into_response(),
        Err(error) => (StatusCode::UNPROCESSABLE_ENTITY, error).into_response(),
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "version": "3.0.1",
    }))]
#[serde(rename_all = "camelCase")]
/// Running API version.
pub(crate) struct ApiVersionResponse {
    #[schema(example = "3.0.1")]
    version: String,
}

/// Returns the API version.
#[utoipa::path(
    get,
    path = "/api_version",
    description="Returns the API version",
    responses(
        (status = 200, description = "API version is returned", body = ApiVersionResponse),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 412, description = "The node is not started and running"),
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Meta"
)]
pub(super) async fn api_version(State(_state): State<Arc<InternalState>>) -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION").to_owned();
    (StatusCode::OK, Json(ApiVersionResponse { version })).into_response()
}
