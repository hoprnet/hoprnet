use std::sync::Arc;

use axum::{extract::State, http::status::StatusCode, response::IntoResponse};
use hopr_lib::api::{
    network::{Health, NetworkView},
    node::{HasChainApi, HasNetworkView, HoprNodeOperations, HoprState},
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
    eval_precondition(is_running(&state))
}

/// Check whether the node is **ready** to accept connections.
///
/// Ready means that the node is running, has at least minimal connectivity,
/// and the chain connector is available.
///
/// # Behavior
///
/// All conditions must be true for 200 OK:
/// 1. Node must be in Running state (`HoprState::Running`)
/// 2. Network must be minimally connected (`Health::Orange`, `Health::Yellow`, or `Health::Green`)
/// 3. Chain connector must not be unavailable
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
pub(super) async fn readyz<H: HoprNodeOperations + HasNetworkView + HasChainApi + Send + Sync + 'static>(
    State(state): State<Arc<AppState<H>>>,
) -> impl IntoResponse {
    eval_precondition(is_running(&state) && is_minimally_connected(&state) && is_chain_available(&state))
}

/// Check whether the node is **healthy**.
///
/// Healthy means that the node is running and has at least minimal connectivity.
///
/// Unlike `readyz`, this endpoint does NOT check chain availability — transient blokli outages
/// must not trigger pod restarts (see #7722).
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
    eval_precondition(is_running(&state) && is_minimally_connected(&state))
}

/// Check if the node has minimal network connectivity.
#[inline]
fn is_minimally_connected<H: HasNetworkView>(state: &AppState<H>) -> bool {
    matches!(
        state.hopr.network_view().health(),
        Health::Orange | Health::Yellow | Health::Green
    )
}

/// A degraded chain is still considered available for readiness purposes.
#[inline]
fn is_chain_available<H: HasChainApi>(state: &AppState<H>) -> bool {
    !HasChainApi::status(&*state.hopr).is_unavailable()
}

/// Check if the node is in the Running state.
#[inline]
fn is_running<H: HoprNodeOperations>(state: &AppState<H>) -> bool {
    matches!(HoprNodeOperations::status(&*state.hopr), HoprState::Running)
}

/// Evaluate a precondition and return the appropriate HTTP response.
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
    use crate::testing::{ChecksNode, MockNodeOps, NoopNode};

    fn startedz_router(mock: MockNodeOps) -> Router {
        Router::new()
            .route("/startedz", get(startedz::<MockNodeOps>))
            .with_state(Arc::new(AppState { hopr: Arc::new(mock) }))
    }

    fn readyz_router(node: ChecksNode) -> Router {
        Router::new()
            .route("/readyz", get(readyz::<ChecksNode>))
            .route("/healthyz", get(healthyz::<ChecksNode>))
            .with_state(Arc::new(AppState { hopr: Arc::new(node) }))
    }

    #[test]
    fn eval_precondition_should_return_ok_when_true() {
        let (parts, _) = eval_precondition(true).into_response().into_parts();
        assert_eq!(parts.status, StatusCode::OK);
    }

    #[test]
    fn eval_precondition_should_return_precondition_failed_when_false() {
        let (parts, _) = eval_precondition(false).into_response().into_parts();
        assert_eq!(parts.status, StatusCode::PRECONDITION_FAILED);
    }

    #[tokio::test]
    async fn startedz_should_return_200_when_running() -> anyhow::Result<()> {
        let mut mock = MockNodeOps::new();
        mock.expect_status().returning(|| HoprState::Running);

        let resp = startedz_router(mock)
            .oneshot(Request::get("/startedz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn startedz_should_return_412_when_not_running() -> anyhow::Result<()> {
        let mut mock = MockNodeOps::new();
        mock.expect_status().returning(|| HoprState::Uninitialized);

        let resp = startedz_router(mock)
            .oneshot(Request::get("/startedz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        Ok(())
    }

    #[tokio::test]
    async fn readyz_should_return_200_when_running_and_connected() -> anyhow::Result<()> {
        let resp = readyz_router(ChecksNode::new(HoprState::Running, Health::Green))
            .oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn readyz_should_return_412_when_running_but_red() -> anyhow::Result<()> {
        let resp = readyz_router(ChecksNode::new(HoprState::Running, Health::Red))
            .oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        Ok(())
    }

    #[tokio::test]
    async fn readyz_should_return_412_when_not_running() -> anyhow::Result<()> {
        let resp = readyz_router(ChecksNode::new(HoprState::WaitingForFunds, Health::Green))
            .oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::PRECONDITION_FAILED);
        Ok(())
    }

    #[tokio::test]
    async fn healthyz_should_return_200_when_running_and_orange() -> anyhow::Result<()> {
        let resp = readyz_router(ChecksNode::new(HoprState::Running, Health::Orange))
            .oneshot(Request::get("/healthyz").body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn eligiblez_should_always_return_200() -> anyhow::Result<()> {
        let app = Router::new()
            .route("/eligiblez", get(eligiblez::<NoopNode>))
            .with_state(Arc::new(AppState {
                hopr: Arc::new(NoopNode),
            }));
        let resp = app.oneshot(Request::get("/eligiblez").body(Body::empty())?).await?;
        assert_eq!(resp.status(), StatusCode::OK);
        Ok(())
    }
}
