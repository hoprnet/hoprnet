use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    Address, HoprTransportError, Multiaddr,
    errors::{HoprLibError, HoprStatusError},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, DurationMilliSeconds, serde_as};
use tracing::debug;

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState};

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
/// Contains the multiaddresses of peers that are `announced` on-chain and `observed` by the node.
pub(crate) struct NodePeerInfoResponse {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19093"]))]
    announced: Vec<Multiaddr>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19093"]))]
    observed: Vec<Multiaddr>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DestinationParams {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    destination: Address,
}

/// Returns transport-related information about the given peer.
///
/// This includes the peer ids that the given peer has `announced` on-chain
/// and peer ids that are actually `observed` by the transport layer.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/peers/{{destination}}"),
    params(
        ("destination" = String, Path, description = "Address of the requested peer", example = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"),
    ),
    responses(
        (status = 200, description = "Peer information fetched successfully.", body = NodePeerInfoResponse),
        (status = 400, description = "Invalid destination", body = ApiError),
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
    Path(DestinationParams { destination }): Path<DestinationParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    match hopr.chain_key_to_peerid(&destination).await {
        Ok(Some(peer)) => Ok((
            StatusCode::OK,
            Json(NodePeerInfoResponse {
                announced: hopr.multiaddresses_announced_on_chain(&peer).await,
                observed: hopr.network_observed_multiaddresses(&peer).await,
            }),
        )),
        Ok(None) => Err(ApiErrorStatus::PeerNotFound),
        Err(_) => Err(ApiErrorStatus::PeerNotFound),
    }
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "latency": 200,
}))]
#[serde(rename_all = "camelCase")]
/// Contains the latency and the reported version of a peer that has been pinged.
pub(crate) struct PingResponse {
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[schema(value_type = u64, example = 200)]
    latency: std::time::Duration,
}

/// Directly pings the given peer.
#[utoipa::path(
    post,
    path = const_format::formatcp!("{BASE_PATH}/peers/{{destination}}/ping"),
    description = "Directly ping the given peer",
    params(
        ("destination" = String, Path, description = "Address of the requested peer", example = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"),
    ),
    responses(
        (status = 200, description = "Ping successful", body = PingResponse),
        (status = 400, description = "Invalid peer id", body = ApiError),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 404, description = "Peer id not found in the network.", body = ApiError),
        (status = 408, description = "Peer timed out.", body = ApiError),
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
    Path(DestinationParams { destination }): Path<DestinationParams>,
    State(state): State<Arc<InternalState>>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(%destination, "Manually ping peer");

    let hopr = state.hopr.clone();

    match hopr.chain_key_to_peerid(&destination).await {
        Ok(Some(peer)) => match hopr.ping(&peer).await {
            Ok((latency, _status)) => {
                let resp = Json(PingResponse { latency: latency / 2 });
                Ok((StatusCode::OK, resp).into_response())
            }
            Err(HoprLibError::TransportError(HoprTransportError::Protocol(hopr_lib::ProtocolError::Timeout))) => {
                Ok((StatusCode::REQUEST_TIMEOUT, ApiErrorStatus::Timeout).into_response())
            }
            Err(HoprLibError::TransportError(HoprTransportError::Probe(
                hopr_lib::ProbeError::ProbeNeighborTimeout(_),
            ))) => Ok((StatusCode::REQUEST_TIMEOUT, ApiErrorStatus::Timeout).into_response()),
            Err(HoprLibError::TransportError(HoprTransportError::Probe(hopr_lib::ProbeError::PingerError(_, e)))) => {
                Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::PingError(e)).into_response())
            }
            Err(HoprLibError::TransportError(HoprTransportError::Probe(hopr_lib::ProbeError::NonExistingPeer))) => {
                Ok((StatusCode::NOT_FOUND, ApiErrorStatus::PeerNotFound).into_response())
            }
            Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(..))) => {
                Ok((StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response())
            }
            Err(e) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response()),
        },
        Ok(None) => Ok((StatusCode::NOT_FOUND, ApiErrorStatus::PeerNotFound).into_response()),
        Err(_) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::PeerNotFound).into_response()),
    }
}
