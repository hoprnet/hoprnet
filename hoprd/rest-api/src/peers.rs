use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{errors::HoprStatusError, HoprTransportError, Multiaddr};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr, DurationMilliSeconds};
use std::{sync::Arc, time::Duration};

use hopr_lib::errors::HoprLibError;

use crate::{types::UnifiedPeerType, ApiError, ApiErrorStatus, InternalState, BASE_PATH};

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "announced": [
        "/ip4/10.0.2.100/tcp/19093"
    ],
    "observed": [
        "/ip4/10.0.2.100/tcp/19093"
    ]
}))]
pub(crate) struct NodePeerInfoResponse {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>)]
    announced: Vec<Multiaddr>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>)]
    observed: Vec<Multiaddr>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PeerIdParams {
    destination: UnifiedPeerType,
}

/// Returns transport-related information about the given peer.
///
/// This includes the peer ids that the given peer has `announced` on-chain
/// and peer ids that are actually `observed` by the transport layer.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/peers/{{peerId}}"),
    params(
        ("peerId" = String, Path, description = "PeerID of the requested peer")
    ),
    responses(
        (status = 200, description = "Peer information fetched successfully.", body = NodePeerInfoResponse),
        (status = 400, description = "Invalid peer id", body = ApiError),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 422, description = "Unknown failure", body = ApiError)
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Peers",
)]
pub(super) async fn show_peer_info(
    Path(PeerIdParams { destination }): Path<PeerIdParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();
    (
        StatusCode::OK,
        Json(NodePeerInfoResponse {
            announced: hopr.multiaddresses_announced_on_chain(&destination.peer_id).await,
            observed: hopr.network_observed_multiaddresses(&destination.peer_id).await,
        }),
    )
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "latency": 200,
    "reportedVersion": "2.1.0"
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct PingResponse {
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[schema(value_type = u64)]
    latency: std::time::Duration,
    reported_version: String,
}

/// Directly pings the given peer.
#[utoipa::path(
    post,
    path = const_format::formatcp!("{BASE_PATH}/peers/{{peerId}}/ping"),
    params(
        ("peerId" = String, Path, description = "PeerID of the requested peer")
    ),
    responses(
        (status = 200, description = "Ping successful", body = PingResponse),
        (status = 400, description = "Invalid peer id", body = ApiError),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 412, description = "The node is not ready."),
        (status = 422, description = "Unknown failure", body = ApiError)
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Peers",
)]
pub(super) async fn ping_peer(
    Path(PeerIdParams { destination }): Path<PeerIdParams>,
    State(state): State<Arc<InternalState>>,
) -> Result<impl IntoResponse, ApiError> {
    let hopr = state.hopr.clone();

    match hopr.ping(&destination.peer_id).await {
        Ok(latency) => {
            let info = hopr.network_peer_info(&destination.peer_id).await?;
            let resp = Json(PingResponse {
                latency: latency.unwrap_or(Duration::ZERO), // TODO: what should be the correct default ?
                reported_version: info.and_then(|p| p.peer_version).unwrap_or("unknown".into()),
            });
            Ok((StatusCode::OK, resp).into_response())
        }
        Err(HoprLibError::TransportError(HoprTransportError::Protocol(hopr_lib::ProtocolError::Timeout))) => {
            Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::Timeout).into_response())
        }
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(_, _))) => {
            Ok((StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response())
        }
        Err(e) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response()),
    }
}
