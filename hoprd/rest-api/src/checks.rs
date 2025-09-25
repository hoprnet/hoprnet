use std::sync::Arc;

use axum::{extract::State, http::status::StatusCode, response::IntoResponse};
use hopr_lib::state::HoprState;

use crate::AppState;

/// Check whether the node is started.
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
    is_running(state) // FIXME: improve this once node state granularity is improved
}

/// Check whether the node is ready to accept connections.
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
    is_running(state) // FIXME: improve this once node state granularity is improved
}

/// Check whether the node is healthy.
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
    is_running(state) // FIXME: improve this once node state granularity is improved
}

fn is_running(state: Arc<AppState>) -> impl IntoResponse {
    match state.hopr.status() {
        HoprState::Running => (StatusCode::OK, "").into_response(),
        _ => (StatusCode::PRECONDITION_FAILED, "").into_response(),
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
