use std::sync::Arc;

use axum::{extract::State, http::status::StatusCode, response::IntoResponse};
use hopr_lib::{Health, state::HoprState};

use crate::AppState;

/// Check whether the node is started.
///
/// # Behavior
///
/// Returns 200 OK when the node is in `HoprState::Running`.
/// Returns 412 PRECONDITION_FAILED when the node is in any other state
/// (Uninitialized, Initializing, Indexing, Starting).
///
/// This endpoint checks only the running state, not network connectivity.
#[utoipa::path(
        get,
        path = "/startedz",
        description="Check whether the node is started",
        responses(
            (status = 200, description = "The node is started and running"),
            (status = 412, description = "The node is not started and running"),
        ),
        tag = "Checks"
    )]
pub(super) async fn startedz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    eval_precondition(is_running(state)) // FIXME: improve this once node state granularity is improved
}

/// Check whether the node is **ready** to accept connections.
///
/// Ready means that the node is running and has at least minimal connectivity.
///
/// # Behavior
///
/// Both conditions must be true for 200 OK:
/// 1. Node must be in Running state (`HoprState::Running`)
/// 2. Network must be minimally connected (`Health::Orange`, `Health::Yellow`, or `Health::Green`)
///
/// Returns 412 PRECONDITION_FAILED if either condition is false:
/// - Node not running (any other `HoprState`)
/// - Node running but network not minimally connected (`Health::Unknown` or `Health::Red`)
///
/// This endpoint is used by Kubernetes readiness probes to determine if the pod should receive traffic.
#[utoipa::path(
        get,
        path = "/readyz",
        description="Check whether the node is ready to accept connections",
        responses(
            (status = 200, description = "The node is ready to accept connections"),
            (status = 412, description = "The node is not ready to accept connections"),
        ),
        tag = "Checks"
    )]
pub(super) async fn readyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    eval_precondition(is_running(state.clone()) && is_minimally_connected(state).await)
}

/// Check whether the node is **healthy**.
///
/// Healthy means that the node is running and has at least minimal connectivity.
///
/// # Behavior
///
/// Both conditions must be true for 200 OK:
/// 1. Node must be in Running state (`HoprState::Running`)
/// 2. Network must be minimally connected (`Health::Orange`, `Health::Yellow`, or `Health::Green`)
///
/// Returns 412 PRECONDITION_FAILED if either condition is false:
/// - Node not running (any other `HoprState`)
/// - Node running but network not minimally connected (`Health::Unknown` or `Health::Red`)
///
/// This endpoint is used by Kubernetes liveness probes to determine if the pod should be restarted.
///
/// Note: Currently `healthyz` and `readyz` have identical behavior.
#[utoipa::path(
        get,
        path = "/healthyz",
        description="Check whether the node is healthy",
        responses(
            (status = 200, description = "The node is healthy"),
            (status = 412, description = "The node is not healthy"),
        ),
        tag = "Checks"
    )]
pub(super) async fn healthyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    eval_precondition(is_running(state.clone()) && is_minimally_connected(state).await)
}

/// Check if the node has minimal network connectivity.
///
/// Returns `true` if the network health is `Orange`, `Yellow`, or `Green`.
/// Returns `false` if the network health is `Unknown` or `Red`.
#[inline]
async fn is_minimally_connected(state: Arc<AppState>) -> bool {
    matches!(
        state.hopr.network_health().await,
        Health::Orange | Health::Yellow | Health::Green
    )
}

/// Check if the node is in the Running state.
///
/// Returns `true` only when `HoprState::Running`.
/// Returns `false` for all other states (Uninitialized, Initializing, Indexing, Starting).
#[inline]
fn is_running(state: Arc<AppState>) -> bool {
    matches!(state.hopr.status(), HoprState::Running)
}

/// Evaluate a precondition and return the appropriate HTTP response.
///
/// Returns 200 OK if `precondition` is `true`.
/// Returns 412 PRECONDITION_FAILED if `precondition` is `false`.
#[inline]
fn eval_precondition(precondition: bool) -> impl IntoResponse {
    if precondition {
        (StatusCode::OK, "").into_response()
    } else {
        (StatusCode::PRECONDITION_FAILED, "").into_response()
    }
}

/// Check whether the node is eligible in the network.
#[utoipa::path(
        get,
        path = "/eligiblez",
        description="Check whether the node is eligible in the network",
        responses(
            (status = 200, description = "The node is allowed in the network"),
            (status = 412, description = "The node is not allowed in the network"),
            (status = 500, description = "Internal server error"),
        ),
        tag = "Checks"
    )]
pub(super) async fn eligiblez(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.get_eligibility_status().await {
        Ok(true) => (StatusCode::OK, "").into_response(),
        Ok(false) => (StatusCode::PRECONDITION_FAILED, "Node not eligible").into_response(),
        Err(hopr_lib::errors::HoprLibError::ChainApi(e)) => {
            // The "division by zero" error is caused by the self-registration,
            // which is forbidden to the public and thus returns false
            // therefore the eligibility check should be ignored
            let err_str = e.to_string();
            if err_str.to_lowercase().contains("division or modulo by zero") {
                (StatusCode::PRECONDITION_FAILED, "Node not eligible").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, err_str).into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that eval_precondition returns 200 OK when the precondition is true
    #[test]
    fn test_eval_precondition_true_returns_ok() {
        let response = eval_precondition(true);
        let (parts, _) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::OK);
    }

    /// Test that eval_precondition returns 412 PRECONDITION_FAILED when the precondition is false
    #[test]
    fn test_eval_precondition_false_returns_precondition_failed() {
        let response = eval_precondition(false);
        let (parts, _) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::PRECONDITION_FAILED);
    }
}
