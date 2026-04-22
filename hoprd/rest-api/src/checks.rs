use std::sync::Arc;

use axum::{extract::State, http::status::StatusCode, response::IntoResponse};
use hopr_lib::api::{
    network::{Health, NetworkView},
    node::{HasNetworkView, HoprNodeOperations, HoprState},
};

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
pub(super) async fn startedz<H: HoprNodeOperations + Send + Sync + 'static>(
    State(state): State<Arc<AppState<H>>>,
) -> impl IntoResponse {
    eval_precondition(is_running(state))
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
pub(super) async fn readyz<H: HoprNodeOperations + HasNetworkView + Send + Sync + 'static>(
    State(state): State<Arc<AppState<H>>>,
) -> impl IntoResponse {
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
pub(super) async fn healthyz<H: HoprNodeOperations + HasNetworkView + Send + Sync + 'static>(
    State(state): State<Arc<AppState<H>>>,
) -> impl IntoResponse {
    eval_precondition(is_running(state.clone()) && is_minimally_connected(state).await)
}

/// Check if the node has minimal network connectivity.
///
/// Returns `true` if the network health is `Orange`, `Yellow`, or `Green`.
/// Returns `false` if the network health is `Unknown` or `Red`.
#[inline]
async fn is_minimally_connected<H: HasNetworkView + Send + Sync + 'static>(state: Arc<AppState<H>>) -> bool {
    matches!(
        state.hopr.network_view().health(),
        Health::Orange | Health::Yellow | Health::Green
    )
}

/// Check if the node is in the Running state.
///
/// Returns `true` only when `HoprState::Running`.
/// Returns `false` for all other states (Uninitialized, Initializing, Indexing, Starting).
#[inline]
fn is_running<H: HoprNodeOperations>(state: Arc<AppState<H>>) -> bool {
    matches!(HoprNodeOperations::status(&*state.hopr), HoprState::Running)
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
pub(super) async fn eligiblez<H: Send + Sync + 'static>(State(_state): State<Arc<AppState<H>>>) -> impl IntoResponse {
    (StatusCode::OK, "").into_response()
}

#[cfg(test)]
mod tests {
    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    use super::*;
    use crate::testing::StubNode;

    fn checks_router(stub: StubNode) -> Router {
        let state: Arc<AppState<StubNode>> = Arc::new(AppState {
            hopr: Arc::new(stub),
        });
        Router::new()
            .route("/startedz", get(startedz::<StubNode>))
            .route("/readyz", get(readyz::<StubNode>))
            .route("/healthyz", get(healthyz::<StubNode>))
            .route("/eligiblez", get(eligiblez::<StubNode>))
            .with_state(state)
    }

    #[test]
    fn eval_precondition_should_return_ok_when_true() {
        let response = eval_precondition(true);
        let (parts, _) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::OK);
    }

    #[test]
    fn eval_precondition_should_return_precondition_failed_when_false() {
        let response = eval_precondition(false);
        let (parts, _) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::PRECONDITION_FAILED);
    }

    #[tokio::test]
    async fn startedz_should_return_200_when_running() -> anyhow::Result<()> {
        let app = checks_router(StubNode::running_and_healthy());
        let resp = app
            .oneshot(Request::get("/startedz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn startedz_should_return_412_when_not_running() -> anyhow::Result<()> {
        let app = checks_router(StubNode::running_and_healthy().with_state(HoprState::Uninitialized));
        let resp = app
            .oneshot(Request::get("/startedz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        Ok(())
    }

    #[tokio::test]
    async fn readyz_should_return_200_when_running_and_connected() -> anyhow::Result<()> {
        let app = checks_router(StubNode::running_and_healthy());
        let resp = app
            .oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn readyz_should_return_412_when_running_but_red() -> anyhow::Result<()> {
        let app = checks_router(StubNode::running_and_healthy().with_health(Health::Red));
        let resp = app
            .oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        Ok(())
    }

    #[tokio::test]
    async fn readyz_should_return_412_when_not_running() -> anyhow::Result<()> {
        let app = checks_router(StubNode::running_and_healthy().with_state(HoprState::WaitingForFunds));
        let resp = app
            .oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        Ok(())
    }

    #[tokio::test]
    async fn healthyz_should_return_200_when_running_and_orange() -> anyhow::Result<()> {
        let app = checks_router(StubNode::running_and_healthy().with_health(Health::Orange));
        let resp = app
            .oneshot(Request::get("/healthyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn eligiblez_should_always_return_200() -> anyhow::Result<()> {
        let app = checks_router(StubNode::running_and_healthy());
        let resp = app
            .oneshot(Request::get("/eligiblez").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }
}
