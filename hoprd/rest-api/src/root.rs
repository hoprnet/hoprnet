use axum::{http::StatusCode, response::IntoResponse};

use crate::{ApiError, ApiErrorStatus};

#[cfg(feature = "telemetry")]
fn collect_hopr_metrics() -> Result<String, ApiErrorStatus> {
    hopr_metrics::gather_all_metrics()
        .map(|metrics| {
            metrics
                .lines()
                .filter(|line| {
                    !(line.starts_with("hopr_session_")
                        || line.starts_with("# HELP hopr_session_")
                        || line.starts_with("# TYPE hopr_session_"))
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .map_err(|_| ApiErrorStatus::UnknownFailure("Failed to gather metrics".into()))
}

#[cfg(not(feature = "telemetry"))]
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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use axum::{body::to_bytes, response::IntoResponse};

    use super::*;

    #[cfg(feature = "telemetry")]
    #[tokio::test]
    async fn collect_metrics_filters_out_session_metrics() -> Result<()> {
        let session_metric_name = "hopr_session_metrics_endpoint_test".to_string();
        let non_session_metric_name = "hopr_metrics_endpoint_test".to_string();

        let session_metric =
            hopr_metrics::MultiCounter::new(&session_metric_name, "session endpoint filtering test", &["session_id"])?;

        let non_session_metric =
            hopr_metrics::MultiCounter::new(&non_session_metric_name, "endpoint non-session metric test", &["kind"])?;

        session_metric.increment(&["test-session"]);
        non_session_metric.increment(&["test-kind"]);

        let collected_metrics = collect_hopr_metrics()
            .map_err(|error| anyhow::anyhow!("collect_hopr_metrics should return metrics: {error}"))?;

        assert!(!collected_metrics.contains(&session_metric_name));
        assert!(collected_metrics.contains(&non_session_metric_name));

        let body = to_bytes(metrics().await.into_response().into_body(), usize::MAX).await?;
        let body_text = String::from_utf8(body.to_vec())?;

        assert!(!body_text.contains(&session_metric_name),);
        assert!(body_text.contains(&non_session_metric_name));

        Ok(())
    }
}
