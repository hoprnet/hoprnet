use axum::{extract::State, http::status::StatusCode, response::IntoResponse};
use std::sync::Arc;

use hopr_lib::HoprState;

use crate::AppState;

/// Check whether the node is started.
#[utoipa::path(
        get,
        path = "/startedz",
        responses(
            (status = 200, description = "The node is stared and running"),
            (status = 412, description = "The node is not started and running"),
        ),
        tag = "Checks"
    )]
pub(super) async fn startedz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    is_running(state)
}

/// Check whether the node is ready to accept connections.
#[utoipa::path(
        get,
        path = "/readyz",
        responses(
            (status = 200, description = "The node is ready to accept connections"),
            (status = 412, description = "The node is not ready to accept connections"),
        ),
        tag = "Checks"
    )]
pub(super) async fn readyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    is_running(state)
}

/// Check whether the node is healthy.
#[utoipa::path(
        get,
        path = "/healthyz",
        responses(
            (status = 200, description = "The node is healthy"),
            (status = 412, description = "The node is not healthy"),
        ),
        tag = "Checks"
    )]
pub(super) async fn healthyz(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    is_running(state)
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
        responses(
            (status = 200, description = "The node is allowed in the network"),
            (status = 412, description = "The node is not allowed in the network"),
        ),
        tag = "Checks"
    )]
pub(super) async fn eligiblez(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.get_eligibility_status().await {
        Ok(true) => (StatusCode::OK, "").into_response(),
        _ => (StatusCode::PRECONDITION_FAILED, "").into_response(),
    }
}
