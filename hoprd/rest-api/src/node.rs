use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Json, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::{StreamExt, stream::FuturesUnordered};
use hopr_lib::{
    Address, Multiaddr,
    api::{
        graph::Observable,
        network::Health,
        node::{HoprNodeChainOperations, HoprNodeNetworkOperations},
    },
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState, checksum_address_serializer};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "version": "2.1.0",
    }))]
#[serde(rename_all = "camelCase")]
/// Running node version.
pub(crate) struct NodeVersionResponse {
    #[schema(example = "2.1.0")]
    version: String,
}

/// Get the release version of the running node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/version"),
        description = "Get the release version of the running node",
        responses(
            (status = 200, description = "Fetched node version", body = NodeVersionResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Node"
    )]
pub(super) async fn version() -> impl IntoResponse {
    let version = hopr_lib::constants::APP_VERSION.to_string();
    (StatusCode::OK, Json(NodeVersionResponse { version })).into_response()
}

/// Get the configuration of the running node.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/node/configuration"),
    description = "Get the configuration of the running node",
    responses(
        (status = 200, description = "Fetched node configuration", body = HashMap<String, String>, example = json!({
        "network": "anvil-localhost",
        "provider": "http://127.0.0.1:8545",
        "hoprToken": "0x9a676e781a523b5d0c0e43731313a708cb607508",
        "hoprChannels": "0x9a9f2ccfde556a7e9ff0848998aa4a0cfd8863ae",
        "...": "..."
        })),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Configuration"
    )]
pub(super) async fn configuration(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.hoprd_cfg.clone())).into_response()
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[schema(example = json!({
        "quality": 0.7
    }))]
/// Quality information for a peer.
pub(crate) struct NodePeersQueryRequest {
    #[serde(default)]
    #[schema(required = false, example = 0.7)]
    /// Minimum peer quality to be included in the response.
    score: f64,
}

#[derive(Debug, Default, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "sent": 10,
    "success": 10
}))]
#[serde(rename_all = "camelCase")]
/// Heartbeat information for a peer.
pub(crate) struct HeartbeatInfo {
    #[schema(example = 10)]
    sent: u64,
    #[schema(example = 10)]
    success: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    "multiaddr": "/ip4/178.12.1.9/tcp/19092",
    "probeRate": 0.476,
    "lastSeen": 1690000000,
    "averageLatency": 100,
    "score": 0.7,
}))]
/// All information about a known peer.
pub(crate) struct PeerObservations {
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Address,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = Option<String>, example = "/ip4/178.12.1.9/tcp/19092")]
    multiaddr: Option<Multiaddr>,
    #[schema(example = 0.476)]
    probe_rate: f64,
    #[schema(example = 1690000000)]
    last_update: u128,
    #[schema(example = 100)]
    average_latency: u128,
    #[schema(example = 0.7)]
    score: f64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    "multiaddrs": "[/ip4/178.12.1.9/tcp/19092]"
}))]
#[serde(rename_all = "camelCase")]
/// Represents a peer that has been announced on-chain.
pub(crate) struct AnnouncedPeer {
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Address,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = "[/ip4/178.12.1.9/tcp/19092]")]
    multiaddrs: Vec<Multiaddr>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
    "connected": [{
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "multiaddr": "/ip4/178.12.1.9/tcp/19092",
        "heartbeats": {
            "sent": 10,
            "success": 10
        },
        "lastSeen": 1690000000,
        "lastSeenLatency": 100,
        "quality": 0.7,
        "backoff": 0.5,
        "isNew": true,
    }],
    "announced": [{
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "multiaddr": "/ip4/178.12.1.9/tcp/19092"
    }]
}))]
/// All connected and announced peers.
pub(crate) struct NodePeersResponse {
    #[schema(example = json!([{
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "multiaddr": "/ip4/178.12.1.9/tcp/19092",
        "heartbeats": {
            "sent": 10,
            "success": 10
        },
        "lastSeen": 1690000000,
        "lastSeenLatency": 100,
        "quality": 0.7,
        "backoff": 0.5,
        "isNew": true,
    }]))]
    connected: Vec<PeerObservations>,
    #[schema(example = json!([{
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "multiaddr": "/ip4/178.12.1.9/tcp/19092"
    }]))]
    announced: Vec<AnnouncedPeer>,
}

/// Lists information for `connected peers` and `announced peers`.
///
/// Connected peers are nodes which are connected to the node while announced peers are
/// nodes which have announced to the network.
///
/// Optionally pass `quality` parameter to get only peers with higher or equal quality
/// to the specified value.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/peers"),
        description = "Lists information for connected and announced peers",
        params(NodePeersQueryRequest),
        responses(
            (status = 200, description = "Successfully returned observed peers", body = NodePeersResponse),
            (status = 400, description = "Failed to extract a valid quality parameter", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Node"
    )]
pub(super) async fn peers(
    Query(NodePeersQueryRequest { score }): Query<NodePeersQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> Result<impl IntoResponse, ApiError> {
    if !(0.0f64..=1.0f64).contains(&score) {
        return Ok((StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidQuality).into_response());
    }

    let hopr = state.hopr.clone();

    let all_network_peers = futures::stream::iter(hopr.network_connected_peers().await?)
        .filter_map(|peer| {
            let hopr = hopr.clone();

            async move {
                if let Some(info) = hopr.network_peer_info(&peer) {
                    if info.score() >= score {
                        Some((peer, info))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        })
        .filter_map(|(peer_id, info)| {
            let hopr = hopr.clone();

            async move {
                let address = hopr.peerid_to_chain_key(&peer_id).await.ok().flatten();

                // WARNING: Only in Providence and Saint-Louis are all peers public
                let multiaddresses = hopr.network_observed_multiaddresses(&peer_id).await;

                Some((address, multiaddresses, info))
            }
        })
        // Filter out peers without a known chain address
        .filter_map(|(address, mas, info)| async move { address.map(|addr| (addr, mas, info)) })
        .map(|(address, mas, info)| PeerObservations {
            address,
            multiaddr: mas.first().cloned(),
            last_update: info.last_update().as_millis(),
            average_latency: info.average_latency().map_or(0, |d| d.as_millis()),
            probe_rate: info.average_probe_rate(),
            score: info.score(),
        })
        .collect::<Vec<_>>()
        .await;

    let announced_peers = hopr
        .accounts_announced_on_chain()
        .await?
        .into_iter()
        .map(|announced| async move {
            AnnouncedPeer {
                address: announced.chain_addr,
                multiaddrs: announced.get_multiaddrs().to_vec(),
            }
        })
        .collect::<FuturesUnordered<_>>()
        .collect()
        .await;

    let body = NodePeersResponse {
        connected: all_network_peers,
        announced: announced_peers,
    };

    Ok((StatusCode::OK, Json(body)).into_response())
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "announcedAddress": [
            "/ip4/10.0.2.100/tcp/19092"
        ],
        "providerUrl": "https://staging.blokli.hoprnet.link",
        "hoprNetworkName": "rotsee",
        "channelClosurePeriod": 15,
        "connectivityStatus": "Green",
        "hoprNodeSafe": "0x42bc901b1d040f984ed626eff550718498a6798a",
        "listeningAddress": [
            "/ip4/10.0.2.100/tcp/19092"
        ],
    }))]
#[serde(rename_all = "camelCase")]
/// Information about the current node. Covers network, addresses, eligibility, connectivity status, contracts addresses
/// and indexer state.
pub(crate) struct NodeInfoResponse {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19092"]))]
    announced_address: Vec<Multiaddr>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19092"]))]
    listening_address: Vec<Multiaddr>,
    #[schema(value_type = String, example = "https://staging.blokli.hoprnet.link")]
    provider_url: String,
    #[schema(value_type = String, example = "rotsee")]
    hopr_network_name: String,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x42bc901b1d040f984ed626eff550718498a6798a")]
    hopr_node_safe: Address,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "Green")]
    connectivity_status: Health,
    /// Channel closure period in seconds
    #[schema(example = 15)]
    channel_closure_period: u64,
}

/// Get information about this HOPR Node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/info"),
        description = "Get information about this HOPR Node",
        responses(
            (status = 200, description = "Fetched node informations", body = NodeInfoResponse),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Node"
    )]
pub(super) async fn info(State(state): State<Arc<InternalState>>) -> Result<impl IntoResponse, ApiError> {
    let hopr = state.hopr.clone();

    let safe_config = hopr.get_safe_config();

    let provider_url = state
        .hoprd_cfg
        .as_object()
        .and_then(|cfg| cfg.get("blokli_url"))
        .and_then(|v| v.as_str());

    match futures::try_join!(hopr.chain_info(), hopr.get_channel_closure_notice_period()) {
        Ok((info, channel_closure_notice_period)) => {
            let body = NodeInfoResponse {
                announced_address: hopr.local_multiaddresses(),
                listening_address: hopr.local_multiaddresses(),
                provider_url: provider_url.unwrap_or("n/a").to_owned(),
                hopr_network_name: info.hopr_network_name,
                hopr_node_safe: safe_config.safe_address,
                connectivity_status: hopr.network_health().await,
                channel_closure_period: channel_closure_notice_period.as_secs(),
            };

            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(error) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
        "isEligible": true,
        "multiaddrs": ["/ip4/10.0.2.100/tcp/19091"]
}))]
/// Reachable entry node information
pub(crate) struct EntryNode {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19091"]))]
    multiaddrs: Vec<Multiaddr>,
    #[schema(example = true)]
    is_eligible: bool,
}

/// List all known entry nodes with multiaddrs and eligibility.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/entry-nodes"),
        description = "List all known entry nodes with multiaddrs and eligibility",
        responses(
            (status = 200, description = "Fetched public nodes' information", body = HashMap<String, EntryNode>, example = json!({
                "0x188c4462b75e46f0c7262d7f48d182447b93a93c": {
                    "isEligible": true,
                    "multiaddrs": ["/ip4/10.0.2.100/tcp/19091"]
                }
            })),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Node"
    )]
pub(super) async fn entry_nodes(State(state): State<Arc<InternalState>>) -> Result<impl IntoResponse, ApiError> {
    let hopr = state.hopr.clone();

    match hopr.get_public_nodes().await {
        Ok(nodes) => {
            let mut body = HashMap::new();
            for (_, address, mas) in nodes.into_iter() {
                body.insert(
                    address.to_string(),
                    EntryNode {
                        multiaddrs: mas,
                        is_eligible: true,
                    },
                );
            }

            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(error) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    }
}
