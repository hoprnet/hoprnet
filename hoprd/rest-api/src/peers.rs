use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::FutureExt;
use hopr_lib::{
    Address, Multiaddr,
    api::node::HoprNodeNetworkOperations,
    errors::{HoprLibError, HoprStatusError, HoprTransportError},
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, DurationMilliSeconds, serde_as};
use tracing::debug;

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState, network::AnnouncementOriginResponse};

/// A multiaddress paired with its discovery origin.
#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "multiaddress": "/ip4/10.0.2.100/tcp/19093",
    "origin": "chain"
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct MultiaddressSource {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "/ip4/10.0.2.100/tcp/19093")]
    multiaddress: Multiaddr,
    #[schema(example = "chain")]
    origin: AnnouncementOriginResponse,
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "announced": ["/ip4/10.0.2.100/tcp/19093"],
    "announcedSources": [
        { "multiaddress": "/ip4/10.0.2.100/tcp/19093", "origin": "chain" }
    ],
    "observed": ["/ip4/10.0.2.100/tcp/19093"]
}))]
#[serde(rename_all = "camelCase")]
/// Contains the multiaddresses of peers that are `announced` on-chain and `observed` by the node.
pub(crate) struct NodePeerInfoResponse {
    /// Flat list of announced multiaddresses (legacy, for backward compatibility).
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19093"]))]
    announced: Vec<Multiaddr>,
    /// Announced multiaddresses grouped by discovery origin.
    announced_sources: Vec<MultiaddressSource>,
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

    match hopr.chain_key_to_peerid(&destination) {
        Ok(Some(peer)) => {
            let res = futures::try_join!(
                hopr.multiaddresses_announced_on_chain(&peer),
                hopr.network_observed_multiaddresses(&peer).map(Ok)
            );
            match res {
                Ok((announced, observed)) => {
                    let announced_sources: Vec<MultiaddressSource> = announced
                        .iter()
                        .map(|ma| MultiaddressSource {
                            multiaddress: ma.clone(),
                            origin: AnnouncementOriginResponse::Chain,
                        })
                        .collect();
                    Ok((
                        StatusCode::OK,
                        Json(NodePeerInfoResponse {
                            announced,
                            announced_sources,
                            observed,
                        }),
                    ))
                }
                Err(error) => Err(ApiErrorStatus::UnknownFailure(error.to_string())),
            }
        }
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

    match hopr.chain_key_to_peerid(&destination) {
        Ok(Some(peer)) => match hopr.ping(&peer).await {
            Ok((latency, _status)) => {
                let resp = Json(PingResponse { latency: latency / 2 });
                Ok((StatusCode::OK, resp).into_response())
            }
            Err(HoprLibError::TransportError(HoprTransportError::Protocol(
                hopr_lib::errors::ProtocolError::Timeout,
            ))) => Ok((StatusCode::REQUEST_TIMEOUT, ApiErrorStatus::Timeout).into_response()),
            Err(HoprLibError::TransportError(HoprTransportError::Probe(hopr_lib::ProbeError::TrafficError(_)))) => {
                Ok((StatusCode::REQUEST_TIMEOUT, ApiErrorStatus::Timeout).into_response())
            }
            Err(HoprLibError::TransportError(HoprTransportError::Probe(hopr_lib::ProbeError::PingerError(e)))) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiaddress_source_should_serialize_with_single_multiaddress() -> anyhow::Result<()> {
        let source = MultiaddressSource {
            multiaddress: "/ip4/1.2.3.4/tcp/9091".parse()?,
            origin: AnnouncementOriginResponse::Chain,
        };

        let json = serde_json::to_value(&source)?;
        assert_eq!(json["multiaddress"], "/ip4/1.2.3.4/tcp/9091");
        assert_eq!(json["origin"], "chain");
        Ok(())
    }

    #[test]
    fn node_peer_info_response_should_include_both_announced_fields() -> anyhow::Result<()> {
        let ma: Multiaddr = "/ip4/10.0.2.100/tcp/19093".parse()?;
        let response = NodePeerInfoResponse {
            announced: vec![ma.clone()],
            announced_sources: vec![MultiaddressSource {
                multiaddress: ma,
                origin: AnnouncementOriginResponse::Chain,
            }],
            observed: vec!["/ip4/10.0.2.100/tcp/19094".parse()?],
        };

        let json = serde_json::to_value(&response)?;
        assert!(json["announced"].is_array());
        assert_eq!(json["announced"][0], "/ip4/10.0.2.100/tcp/19093");
        assert!(json["announcedSources"].is_array());
        assert_eq!(json["announcedSources"][0]["multiaddress"], "/ip4/10.0.2.100/tcp/19093");
        assert_eq!(json["announcedSources"][0]["origin"], "chain");
        assert!(json["observed"].is_array());
        Ok(())
    }

    #[test]
    fn node_peer_info_response_should_serialize_empty_sources_when_no_announcements() -> anyhow::Result<()> {
        let response = NodePeerInfoResponse {
            announced: vec![],
            announced_sources: vec![],
            observed: vec!["/ip4/10.0.2.100/tcp/19094".parse()?],
        };

        let json = serde_json::to_value(&response)?;
        assert_eq!(json["announced"].as_array().unwrap().len(), 0);
        assert_eq!(json["announcedSources"].as_array().unwrap().len(), 0);
        Ok(())
    }
}
