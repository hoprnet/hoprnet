use axum::{http::StatusCode, response::IntoResponse};

use crate::{ApiError, ApiErrorStatus};

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
