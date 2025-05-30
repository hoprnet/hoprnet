use axum::{extract::Request, middleware::Next, response::Response};
use hopr_lib::AsUnixTimestamp;
#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, MultiHistogram, SimpleGauge};
use hopr_platform::time::native::current_time;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_API_CALLS: MultiCounter = MultiCounter::new(
        "hopr_http_api_call_count",
        "Number of different REST API calls and their statuses",
        &["endpoint", "method", "status"]
    )
    .unwrap();
    static ref METRIC_COUNT_API_CALLS_TIMING: MultiHistogram = MultiHistogram::new(
        "hopr_http_api_call_timing_sec",
        "Timing of different REST API calls in seconds",
        vec![0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0],
        &["endpoint", "method"]
    )
    .unwrap();
    static ref METRIC_API_LAST_TIME: SimpleGauge = SimpleGauge::new(
        "hopr_http_api_last_used_time",
        "The unix timestamp in seconds at which any API endpoint was last fetched"
    ).unwrap();

    // Matches Ed25519-based peer IDs and channel IDs (Keccak256 hashes)
    static ref ID_REGEX: regex::Regex = regex::Regex::new(r"(0x[0-9A-Fa-f]{64})|(12D3KooW[A-z0-9]{44})").unwrap();
}

/// Custom prometheus recording middleware
#[cfg(all(feature = "prometheus", not(test)))]
pub(crate) async fn record(
    uri: axum::extract::OriginalUri,
    method: axum::http::Method,
    request: Request,
    next: Next,
) -> Response {
    let path = uri.path().to_owned();

    let start = std::time::Instant::now();
    let response: Response = next.run(request).await;
    let response_duration = start.elapsed();

    let status = response.status();

    // We're not interested in metrics for other than our own API endpoints
    if path.starts_with("/api/v4/") && !path.contains("node/metrics") {
        let path = ID_REGEX.replace(&path, "<id>");
        METRIC_COUNT_API_CALLS.increment(&[&path, method.as_str(), &status.to_string()]);
        METRIC_COUNT_API_CALLS_TIMING.observe(&[&path, method.as_str()], response_duration.as_secs_f64());
    }

    // Set for any API call
    METRIC_API_LAST_TIME.set(current_time().as_unix_timestamp().as_secs_f64());

    response
}

#[cfg(any(not(feature = "prometheus"), test))]
pub(crate) async fn record(request: Request, next: Next) -> Response {
    next.run(request).await
}
