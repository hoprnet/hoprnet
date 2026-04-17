use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    Address, ChannelEntry, ChannelStatus, HoprBalance, HoprIncentiveOperations, Multiaddr,
    api::{
        chain::{AccountSelector, ChainKeyOperations, ChainReadAccountOperations},
        graph::{EdgeLinkObservable, traits::EdgeObservableRead},
        network::NetworkView,
        node::{HasChainApi, HasNetworkView, HasTransportApi},
    },
    errors::{HoprLibError, HoprTransportError},
    prelude::Hash,
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

/// Channel information for a specific peer.
#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "id": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
    "status": "Open",
    "balance": "10 wxHOPR"
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct PeerChannelInfo {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f")]
    id: Hash,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "Open")]
    status: ChannelStatus,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "10 wxHOPR")]
    balance: HoprBalance,
}

/// QoS observation data for a peer.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "probeRate": 0.476,
    "lastUpdate": 1690000000000_u128,
    "averageLatency": 100,
    "score": 0.7
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct PeerQosInfo {
    #[schema(example = 0.476)]
    probe_rate: f64,
    /// Epoch milliseconds of the last observation update.
    #[schema(example = 1690000000000_u128)]
    last_update: u128,
    /// Average latency in milliseconds, if available.
    #[schema(example = 100)]
    average_latency: Option<u128>,
    #[schema(example = 0.7)]
    score: f64,
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "announcedSources": [
        { "multiaddress": "/ip4/10.0.2.100/tcp/19093", "origin": "chain" }
    ],
    "observed": ["/ip4/10.0.2.100/tcp/19093"],
    "qos": { "probeRate": 0.476, "lastUpdate": 1690000000000_u128, "averageLatency": 100, "score": 0.7 },
    "outgoingChannel": { "id": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f", "status": "Open", "balance": "10 wxHOPR" }
}))]
#[serde(rename_all = "camelCase")]
/// Comprehensive information about a peer: multiaddresses, QoS observations, and channel state.
pub(crate) struct NodePeerInfoResponse {
    /// Announced multiaddresses grouped by discovery origin.
    announced_sources: Vec<MultiaddressSource>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19093"]))]
    observed: Vec<Multiaddr>,
    /// QoS observation data from the network graph, if connected.
    #[serde(skip_serializing_if = "Option::is_none")]
    qos: Option<PeerQosInfo>,
    /// Outgoing channel (this node → peer), if one exists and is not closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    outgoing_channel: Option<PeerChannelInfo>,
    /// Incoming channel (peer → this node), if one exists and is not closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    incoming_channel: Option<PeerChannelInfo>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddressParams {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    address: Address,
}

/// Converts announced multiaddresses to the API response format.
fn to_announced_sources(announced: Vec<Multiaddr>) -> Vec<MultiaddressSource> {
    announced
        .into_iter()
        .map(|ma| MultiaddressSource {
            multiaddress: ma,
            origin: AnnouncementOriginResponse::Chain,
        })
        .collect()
}

/// Extracts [`PeerChannelInfo`] from a channel lookup result, filtering out closed channels.
fn channel_entry_to_peer_info(
    result: Result<Option<ChannelEntry>, hopr_lib::errors::HoprLibError>,
) -> Option<PeerChannelInfo> {
    result
        .ok()
        .flatten()
        .filter(|ch| ch.status != ChannelStatus::Closed)
        .map(|ch| PeerChannelInfo {
            id: *ch.get_id(),
            status: ch.status,
            balance: ch.balance,
        })
}

/// Returns comprehensive information about the given peer.
///
/// Includes announced and observed multiaddresses, QoS observation data from the
/// network graph, and the state of any channels between this node and the peer.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/peers/{{address}}"),
    params(
        ("address" = String, Path, description = "On-chain address of the requested peer", example = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"),
    ),
    responses(
        (status = 200, description = "Peer information fetched successfully.", body = NodePeerInfoResponse),
        (status = 400, description = "Invalid peer address", body = ApiError),
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
    Path(AddressParams { address }): Path<AddressParams>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    let offchain_key = match hopr.chain_api().chain_key_to_packet_key(&address) {
        Ok(Some(key)) => key,
        Ok(None) | Err(_) => return (StatusCode::NOT_FOUND, ApiErrorStatus::PeerNotFound).into_response(),
    };
    let peer: hopr_lib::api::PeerId = offchain_key.into();

    // 1. Multiaddresses (announced + observed)
    use futures::StreamExt;
    let announced: Vec<Multiaddr> = match hopr.chain_api().stream_accounts(AccountSelector {
        offchain_key: Some(offchain_key),
        ..Default::default()
    }) {
        Ok(stream) => {
            stream
                .flat_map(|account| futures::stream::iter(account.get_multiaddrs().to_vec()))
                .collect()
                .await
        }
        Err(error) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::UnknownFailure(error.to_string()),
            )
                .into_response();
        }
    };

    let observed: Vec<Multiaddr> = hopr
        .network_view()
        .multiaddress_of(&peer)
        .map(|set| set.into_iter().collect())
        .unwrap_or_default();

    let announced_sources = to_announced_sources(announced);

    // 2. QoS data from graph
    let qos = {
        let graph = hopr.graph();
        let me_key = graph.me();
        let edges = graph.connected_edges();

        let mut found = None;
        for (src, dst, obs) in &edges {
            if src != me_key {
                continue;
            }
            // Resolve this edge's destination to an on-chain address
            let addr = match hopr.chain_api().packet_key_to_chain_key(dst) {
                Ok(Some(a)) => a,
                _ => continue,
            };
            if addr != address {
                continue;
            }
            if let Some(imm) = obs.immediate_qos() {
                found = Some(PeerQosInfo {
                    probe_rate: imm.average_probe_rate(),
                    last_update: obs.last_update().as_millis(),
                    average_latency: imm.average_latency().map(|l| l.as_millis()),
                    score: obs.score(),
                });
            }
            break;
        }
        found
    };

    // 3. Channel state
    let me = hopr.identity().node_address;
    let outgoing_channel = channel_entry_to_peer_info(hopr.channel(me, address).map_err(HoprLibError::chain));
    let incoming_channel = channel_entry_to_peer_info(hopr.channel(address, me).map_err(HoprLibError::chain));

    (
        StatusCode::OK,
        Json(NodePeerInfoResponse {
            announced_sources,
            observed,
            qos,
            outgoing_channel,
            incoming_channel,
        }),
    )
        .into_response()
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
    path = const_format::formatcp!("{BASE_PATH}/peers/{{address}}/ping"),
    description = "Directly ping the given peer",
    params(
        ("address" = String, Path, description = "On-chain address of the requested peer", example = "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"),
    ),
    responses(
        (status = 200, description = "Ping successful", body = PingResponse),
        (status = 400, description = "Invalid peer address", body = ApiError),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
        (status = 404, description = "Peer address not found in the network.", body = ApiError),
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
    Path(AddressParams { address }): Path<AddressParams>,
    State(state): State<Arc<InternalState>>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(%address, "Manually ping peer");

    let hopr = state.hopr.clone();

    let offchain_key = match hopr.chain_api().chain_key_to_packet_key(&address) {
        Ok(Some(key)) => key,
        Ok(None) => return Ok((StatusCode::NOT_FOUND, ApiErrorStatus::PeerNotFound).into_response()),
        Err(_) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::PeerNotFound).into_response()),
    };

    match hopr.transport().ping(&offchain_key).await {
        Ok((latency, _status)) => {
            let resp = Json(PingResponse { latency: latency / 2 });
            Ok((StatusCode::OK, resp).into_response())
        }
        Err(HoprTransportError::Protocol(hopr_lib::errors::ProtocolError::Timeout)) => {
            Ok((StatusCode::REQUEST_TIMEOUT, ApiErrorStatus::Timeout).into_response())
        }
        Err(HoprTransportError::Probe(hopr_lib::ProbeError::TrafficError(_))) => {
            Ok((StatusCode::REQUEST_TIMEOUT, ApiErrorStatus::Timeout).into_response())
        }
        Err(HoprTransportError::Probe(hopr_lib::ProbeError::PingerError(e))) => {
            Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::PingError(e)).into_response())
        }
        Err(HoprTransportError::Probe(hopr_lib::ProbeError::NonExistingPeer)) => {
            Ok((StatusCode::NOT_FOUND, ApiErrorStatus::PeerNotFound).into_response())
        }
        Err(e) => Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
            .into_response()),
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
    fn node_peer_info_response_should_include_announced_sources() -> anyhow::Result<()> {
        let ma: Multiaddr = "/ip4/10.0.2.100/tcp/19093".parse()?;
        let response = NodePeerInfoResponse {
            announced_sources: vec![MultiaddressSource {
                multiaddress: ma,
                origin: AnnouncementOriginResponse::Chain,
            }],
            observed: vec!["/ip4/10.0.2.100/tcp/19094".parse()?],
            qos: None,
            outgoing_channel: None,
            incoming_channel: None,
        };

        let json = serde_json::to_value(&response)?;
        assert!(json.get("announced").is_none());
        assert!(json["announcedSources"].is_array());
        assert_eq!(json["announcedSources"][0]["multiaddress"], "/ip4/10.0.2.100/tcp/19093");
        assert_eq!(json["announcedSources"][0]["origin"], "chain");
        assert!(json["observed"].is_array());
        Ok(())
    }

    #[test]
    fn node_peer_info_response_should_serialize_empty_sources_when_no_announcements() -> anyhow::Result<()> {
        let response = NodePeerInfoResponse {
            announced_sources: vec![],
            observed: vec!["/ip4/10.0.2.100/tcp/19094".parse()?],
            qos: None,
            outgoing_channel: None,
            incoming_channel: None,
        };

        let json = serde_json::to_value(&response)?;
        assert_eq!(json["announcedSources"].as_array().unwrap().len(), 0);
        Ok(())
    }

    #[test]
    fn node_peer_info_response_should_omit_null_optional_fields() -> anyhow::Result<()> {
        let response = NodePeerInfoResponse {
            announced_sources: vec![],
            observed: vec![],
            qos: None,
            outgoing_channel: None,
            incoming_channel: None,
        };

        let json = serde_json::to_value(&response)?;
        assert!(json.get("qos").is_none());
        assert!(json.get("outgoingChannel").is_none());
        assert!(json.get("incomingChannel").is_none());
        Ok(())
    }

    #[test]
    fn peer_channel_info_should_serialize_correctly() -> anyhow::Result<()> {
        let info = PeerChannelInfo {
            id: Hash::default(),
            status: ChannelStatus::Open,
            balance: "10 wxHOPR".parse()?,
        };

        let json = serde_json::to_value(&info)?;
        assert_eq!(json["status"], "Open");
        assert_eq!(json["balance"], "10 wxHOPR");
        assert!(json.get("id").is_some());
        Ok(())
    }

    #[test]
    fn peer_qos_info_should_serialize_with_all_fields() {
        let qos = PeerQosInfo {
            probe_rate: 0.5,
            last_update: 1690000000000,
            average_latency: Some(100),
            score: 0.7,
        };

        let json = serde_json::to_value(&qos).unwrap();
        assert_eq!(json["probeRate"], 0.5);
        assert_eq!(json["lastUpdate"], 1690000000000_u64);
        assert_eq!(json["averageLatency"], 100);
        assert_eq!(json["score"], 0.7);
    }

    #[test]
    fn peer_qos_info_should_serialize_without_latency() {
        let qos = PeerQosInfo {
            probe_rate: 0.3,
            last_update: 1690000000000,
            average_latency: None,
            score: 0.5,
        };

        let json = serde_json::to_value(&qos).unwrap();
        assert!(json.get("averageLatency").unwrap().is_null());
    }

    #[test]
    fn node_peer_info_response_should_include_populated_optional_fields() -> anyhow::Result<()> {
        let response = NodePeerInfoResponse {
            announced_sources: vec![],
            observed: vec![],
            qos: Some(PeerQosInfo {
                probe_rate: 0.5,
                last_update: 1690000000000,
                average_latency: Some(100),
                score: 0.7,
            }),
            outgoing_channel: Some(PeerChannelInfo {
                id: Hash::default(),
                status: ChannelStatus::Open,
                balance: "10 wxHOPR".parse()?,
            }),
            incoming_channel: Some(PeerChannelInfo {
                id: Hash::default(),
                status: ChannelStatus::Open,
                balance: "5 wxHOPR".parse()?,
            }),
        };

        let json = serde_json::to_value(&response)?;
        assert!(json.get("qos").is_some());
        assert!(json.get("outgoingChannel").is_some());
        assert!(json.get("incomingChannel").is_some());
        assert_eq!(json["qos"]["probeRate"], 0.5);
        assert_eq!(json["outgoingChannel"]["balance"], "10 wxHOPR");
        assert_eq!(json["incomingChannel"]["balance"], "5 wxHOPR");
        Ok(())
    }

    #[test]
    fn ping_response_should_serialize_latency_as_millis() {
        let response = PingResponse {
            latency: std::time::Duration::from_millis(200),
        };

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["latency"], 200);
    }

    #[test]
    fn to_announced_sources_should_convert_multiaddresses() {
        let addrs: Vec<Multiaddr> = vec![
            "/ip4/10.0.2.100/tcp/19093".parse().unwrap(),
            "/ip4/10.0.2.101/tcp/19094".parse().unwrap(),
        ];

        let sources = to_announced_sources(addrs);
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].origin, AnnouncementOriginResponse::Chain);
        assert_eq!(sources[1].origin, AnnouncementOriginResponse::Chain);
    }

    #[test]
    fn to_announced_sources_should_return_empty_for_empty_input() {
        let sources = to_announced_sources(vec![]);
        assert!(sources.is_empty());
    }

    #[test]
    fn channel_entry_to_peer_info_should_return_some_for_open_channel() {
        let ch = ChannelEntry::builder()
            .between(
                "0x07eaf07d6624f741e04f4092a755a9027aaab7f6".parse::<Address>().unwrap(),
                "0x188c4462b75e46f0c7262d7f48d182447b93a93c".parse::<Address>().unwrap(),
            )
            .balance("10 wxHOPR".parse().unwrap())
            .status(ChannelStatus::Open)
            .build()
            .unwrap();

        let info = channel_entry_to_peer_info(Ok(Some(ch)));
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.status, ChannelStatus::Open);
        assert_eq!(info.balance, "10 wxHOPR".parse().unwrap());
    }

    #[test]
    fn channel_entry_to_peer_info_should_return_some_for_pending_to_close() {
        let ch = ChannelEntry::builder()
            .between(
                "0x07eaf07d6624f741e04f4092a755a9027aaab7f6".parse::<Address>().unwrap(),
                "0x188c4462b75e46f0c7262d7f48d182447b93a93c".parse::<Address>().unwrap(),
            )
            .balance("5 wxHOPR".parse().unwrap())
            .status(ChannelStatus::PendingToClose(std::time::SystemTime::now()))
            .build()
            .unwrap();

        assert!(channel_entry_to_peer_info(Ok(Some(ch))).is_some());
    }

    #[test]
    fn channel_entry_to_peer_info_should_return_none_for_closed_channel() {
        let ch = ChannelEntry::builder()
            .between(
                "0x07eaf07d6624f741e04f4092a755a9027aaab7f6".parse::<Address>().unwrap(),
                "0x188c4462b75e46f0c7262d7f48d182447b93a93c".parse::<Address>().unwrap(),
            )
            .balance("10 wxHOPR".parse().unwrap())
            .status(ChannelStatus::Closed)
            .build()
            .unwrap();

        assert!(channel_entry_to_peer_info(Ok(Some(ch))).is_none());
    }

    #[test]
    fn channel_entry_to_peer_info_should_return_none_for_no_channel() {
        assert!(channel_entry_to_peer_info(Ok(None)).is_none());
    }

    #[test]
    fn channel_entry_to_peer_info_should_return_none_for_error() {
        let err = hopr_lib::errors::HoprLibError::GeneralError("test".into());
        assert!(channel_entry_to_peer_info(Err(err)).is_none());
    }

    #[test]
    fn address_params_should_deserialize() {
        let json = serde_json::json!({
            "address": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
        });
        let params: AddressParams = serde_json::from_value(json).unwrap();
        assert_eq!(
            params.address,
            "0x07eaf07d6624f741e04f4092a755a9027aaab7f6".parse().unwrap()
        );
    }

    #[test]
    fn address_params_should_reject_invalid_address() {
        let json = serde_json::json!({ "address": "not-an-address" });
        assert!(serde_json::from_value::<AddressParams>(json).is_err());
    }

    #[test]
    fn peer_channel_info_should_serialize_pending_to_close_status() -> anyhow::Result<()> {
        let info = PeerChannelInfo {
            id: Hash::default(),
            status: ChannelStatus::PendingToClose(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1)),
            balance: "3 wxHOPR".parse()?,
        };
        let json = serde_json::to_value(&info)?;
        assert!(json["status"].as_str().unwrap().starts_with("PendingToClose"));
        Ok(())
    }

    #[test]
    fn to_announced_sources_should_preserve_order() {
        let addrs: Vec<Multiaddr> = vec![
            "/ip4/10.0.2.100/tcp/19093".parse().unwrap(),
            "/ip4/10.0.2.101/tcp/19094".parse().unwrap(),
            "/ip4/10.0.2.102/tcp/19095".parse().unwrap(),
        ];
        let sources = to_announced_sources(addrs.clone());
        for (i, src) in sources.iter().enumerate() {
            assert_eq!(src.multiaddress, addrs[i]);
        }
    }

    #[test]
    fn multiaddress_source_origin_should_serialize_as_lowercase_string() -> anyhow::Result<()> {
        let source = MultiaddressSource {
            multiaddress: "/ip4/127.0.0.1/tcp/1".parse()?,
            origin: AnnouncementOriginResponse::Chain,
        };
        let json = serde_json::to_value(&source)?;
        assert_eq!(json["origin"], "chain");
        Ok(())
    }

    #[test]
    fn node_peer_info_response_should_flatten_announced_sources_in_json() -> anyhow::Result<()> {
        let response = NodePeerInfoResponse {
            announced_sources: vec![
                MultiaddressSource {
                    multiaddress: "/ip4/10.0.2.100/tcp/19093".parse()?,
                    origin: AnnouncementOriginResponse::Chain,
                },
                MultiaddressSource {
                    multiaddress: "/ip4/10.0.2.100/tcp/19094".parse()?,
                    origin: AnnouncementOriginResponse::Chain,
                },
            ],
            observed: vec![],
            qos: None,
            outgoing_channel: None,
            incoming_channel: None,
        };
        let json = serde_json::to_value(&response)?;
        assert_eq!(json["announcedSources"].as_array().unwrap().len(), 2);
        Ok(())
    }
}
