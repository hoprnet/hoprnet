use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Json, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::{StreamExt, stream::FuturesUnordered};
use hopr_crypto_types::prelude::Hash;
use hopr_lib::{Address, AsUnixTimestamp, GraphExportConfig, Health, Multiaddr};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

use crate::{
    ApiError, ApiErrorStatus, BASE_PATH, InternalState, checksum_address_serializer, option_checksum_address_serializer,
};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "version": "2.1.0",
        "apiVersion": "3.10.0"
    }))]
#[serde(rename_all = "camelCase")]
/// Running node version alongside the API version.
pub(crate) struct NodeVersionResponse {
    #[schema(example = "2.1.0")]
    version: String,
    #[schema(example = "3.10.0")]
    api_version: String,
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
pub(super) async fn version(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let version = state.hopr.version();
    let api_version = env!("CARGO_PKG_VERSION").to_owned();
    (StatusCode::OK, Json(NodeVersionResponse { version, api_version })).into_response()
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
    quality: f64,
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
    "heartbeats": {
        "sent": 10,
        "success": 10
    },
    "lastSeen": 1690000000,
    "lastSeenLatency": 100,
    "quality": 0.7,
    "backoff": 0.5,
    "isNew": true,
    "reportedVersion": "2.1.0"
}))]
/// All information about a known peer.
pub(crate) struct PeerInfo {
    #[serde(serialize_with = "option_checksum_address_serializer")]
    #[schema(value_type = Option<String>, example = "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Option<Address>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = Option<String>, example = "/ip4/178.12.1.9/tcp/19092")]
    multiaddr: Option<Multiaddr>,
    #[schema(example = json!({
        "sent": 10,
        "success": 10
    }))]
    heartbeats: HeartbeatInfo,
    #[schema(example = 1690000000)]
    last_seen: u128,
    #[schema(example = 100)]
    last_seen_latency: u128,
    #[schema(example = 0.7)]
    quality: f64,
    #[schema(example = 0.5)]
    backoff: f64,
    #[schema(example = true)]
    is_new: bool,
    #[schema(example = "2.1.0")]
    reported_version: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    "multiaddr": "/ip4/178.12.1.9/tcp/19092"
}))]
#[serde(rename_all = "camelCase")]
/// Represents a peer that has been announced on-chain.
pub(crate) struct AnnouncedPeer {
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe")]
    address: Address,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = Option<String>, example = "/ip4/178.12.1.9/tcp/19092")]
    multiaddr: Option<Multiaddr>,
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
        "reportedVersion": "2.1.0"
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
        "reportedVersion": "2.1.0"
    }]))]
    connected: Vec<PeerInfo>,
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
    Query(NodePeersQueryRequest { quality }): Query<NodePeersQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> Result<impl IntoResponse, ApiError> {
    if !(0.0f64..=1.0f64).contains(&quality) {
        return Ok((StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidQuality).into_response());
    }

    let hopr = state.hopr.clone();

    let all_network_peers = futures::stream::iter(hopr.network_connected_peers().await?)
        .filter_map(|peer| {
            let hopr = hopr.clone();

            async move {
                if let Ok(Some(info)) = hopr.network_peer_info(&peer).await {
                    let avg_quality = info.get_average_quality();
                    if avg_quality >= quality {
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
        .map(|(address, mas, info)| PeerInfo {
            address,
            multiaddr: mas.first().cloned(),
            heartbeats: HeartbeatInfo {
                sent: info.heartbeats_sent,
                success: info.heartbeats_succeeded,
            },
            last_seen: info.last_seen.as_unix_timestamp().as_millis(),
            last_seen_latency: info.last_seen_latency.as_millis() / 2,
            quality: info.get_average_quality(),
            backoff: info.backoff,
            is_new: info.heartbeats_sent == 0u64,
            reported_version: info.peer_version.unwrap_or("UNKNOWN".to_string()),
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
                multiaddr: announced.get_multiaddr(),
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

#[cfg(all(feature = "prometheus", not(test)))]
fn collect_hopr_metrics() -> Result<String, ApiErrorStatus> {
    hopr_metrics::metrics::gather_all_metrics()
        .map_err(|_| ApiErrorStatus::UnknownFailure("Failed to gather metrics".into()))
}

#[cfg(any(not(feature = "prometheus"), test))]
fn collect_hopr_metrics() -> Result<String, ApiErrorStatus> {
    Err(ApiErrorStatus::UnknownFailure("BUILT WITHOUT METRICS SUPPORT".into()))
}

/// Retrieve Prometheus metrics from the running node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/metrics"),
        description = "Retrieve Prometheus metrics from the running node",
        responses(
            (status = 200, description = "Fetched node metrics", body = String),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Node"
    )]
pub(super) async fn metrics() -> impl IntoResponse {
    match collect_hopr_metrics() {
        Ok(metrics) => (StatusCode::OK, metrics).into_response(),
        Err(error) => (StatusCode::UNPROCESSABLE_ENTITY, error).into_response(),
    }
}

#[derive(Debug, Clone, Deserialize, Default, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
#[serde(default, rename_all = "camelCase")]
#[schema(example = json!({
        "ignoreDisconnectedComponents": true,
        "ignoreNonOpenedChannels": true,
        "only3HopPaths": true,
        "rawGraph": true
    }))]
/// Query parameters for the channel graph export.
pub(crate) struct GraphExportQuery {
    /// If set, nodes that are not connected to this node (via open channels) will not be exported.
    /// This setting automatically implies `ignore_non_opened_channels`.
    #[schema(required = false)]
    #[serde(default)]
    pub ignore_disconnected_components: bool,
    /// Do not export channels that are not in the `Open` state.
    #[schema(required = false)]
    #[serde(default)]
    pub ignore_non_opened_channels: bool,
    /// Show only nodes that are accessible via 3-hops (via open channels) from this node.
    #[schema(required = false)]
    #[serde(default)]
    pub only_3_hop_paths: bool,
    /// Export the entire graph in raw JSON format, that can be later
    /// used to load the graph into e.g., a unit test.
    ///
    /// Note that `ignore_disconnected_components` and `ignore_non_opened_channels` are ignored.
    #[schema(required = false)]
    #[serde(default)]
    pub raw_graph: bool,
}

impl From<GraphExportQuery> for GraphExportConfig {
    fn from(value: GraphExportQuery) -> Self {
        Self {
            ignore_disconnected_components: value.ignore_disconnected_components,
            ignore_non_opened_channels: value.ignore_non_opened_channels,
            only_3_hop_accessible_nodes: value.only_3_hop_paths,
        }
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "graph": "
        ...
        242 -> 381 [ label = 'Open channel 0x82a72e271cdedd56c29e970ced3517ba93b679869c729112b5a56fa08698df8f; stake=100000000000000000 HOPR; score=None; status=open;' ]
        ...",
    }))]
#[serde(rename_all = "camelCase")]
/// Response body for the channel graph export.
pub(crate) struct NodeGraphResponse {
    graph: String,
}

/// Retrieve node's channel graph in DOT or JSON format.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/node/graph"),
    description = "Retrieve node's channel graph in DOT or JSON format",
    params(GraphExportQuery),
    responses(
            (status = 200, description = "Fetched channel graph", body = NodeGraphResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
    ),
    security(
            ("api_token" = []),
            ("bearer_token" = [])
    ),
    tag = "Node"
)]
pub(super) async fn channel_graph(
    State(state): State<Arc<InternalState>>,
    Query(args): Query<GraphExportQuery>,
) -> impl IntoResponse {
    if args.raw_graph {
        match state.hopr.export_raw_channel_graph().await {
            Ok(raw_graph) => (StatusCode::OK, Json(NodeGraphResponse { graph: raw_graph })).into_response(),
            Err(error) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::UnknownFailure(error.to_string()),
            )
                .into_response(),
        }
    } else {
        (StatusCode::OK, state.hopr.export_channel_graph(args.into()).await).into_response()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "announcedAddress": [
            "/ip4/10.0.2.100/tcp/19092"
        ],
        "chain": "anvil-localhost",
        "provider": "http://127.0.0.1:8545",
        "channelClosurePeriod": 15,
        "connectivityStatus": "Green",
        "hoprChannels": "0x9a9f2ccfde556a7e9ff0848998aa4a0cfd8863ae",
        "hoprManagementModule": "0xa51c1fc2f0d1a1b8494ed1fe312d7c3a78ed91c0",
        "hoprNetworkRegistry": "0x3aa5ebb10dc797cac828524e59a333d0a371443c",
        "hoprNodeSafe": "0x42bc901b1d040f984ed626eff550718498a6798a",
        "hoprNodeSafeRegistry": "0x0dcd1bf9a1b36ce34237eeafef220932846bcd82",
        "hoprToken": "0x9a676e781a523b5d0c0e43731313a708cb607508",
        "isEligible": true,
        "listeningAddress": [
            "/ip4/10.0.2.100/tcp/19092"
        ],
        "network": "anvil-localhost",
        "indexerBlock": 123456,
        "indexerChecksum": "0000000000000000000000000000000000000000000000000000000000000000",
        "indexBlockPrevChecksum": 0,
        "indexerLastLogBlock": 123450,
        "indexerLastLogChecksum": "cfde556a7e9ff0848998aa4a9a9f2ccfde556a7e9ff0848998aa4a0cfd8863ae",
    }))]
#[serde(rename_all = "camelCase")]
/// Information about the current node. Covers network, addresses, eligibility, connectivity status, contracts addresses
/// and indexer state.
pub(crate) struct NodeInfoResponse {
    #[schema(value_type = String, example = "anvil-localhost")]
    network: String,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19092"]))]
    announced_address: Vec<Multiaddr>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>, example = json!(["/ip4/10.0.2.100/tcp/19092"]))]
    listening_address: Vec<Multiaddr>,
    #[schema(example = "anvil-localhost")]
    chain: String,
    #[schema(example = "http://127.0.0.1:8545")]
    provider: String,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x9a676e781a523b5d0c0e43731313a708cb607508")]
    hopr_token: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x9a9f2ccfde556a7e9ff0848998aa4a0cfd8863ae")]
    hopr_channels: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x3aa5ebb10dc797cac828524e59a333d0a371443c")]
    hopr_network_registry: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x0dcd1bf9a1b36ce34237eeafef220932846bcd82")]
    hopr_node_safe_registry: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0xa51c1fc2f0d1a1b8494ed1fe312d7c3a78ed91c0")]
    hopr_management_module: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String, example = "0x42bc901b1d040f984ed626eff550718498a6798a")]
    hopr_node_safe: Address,
    #[schema(example = true)]
    is_eligible: bool,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "Green")]
    connectivity_status: Health,
    /// Channel closure period in seconds
    #[schema(example = 15)]
    channel_closure_period: u64,
    #[schema(example = 123456)]
    indexer_block: u32,
    #[schema(example = 123450)]
    indexer_last_log_block: u32,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "cfde556a7e9ff0848998aa4a9a9f2ccfde556a7e9ff0848998aa4a0cfd8863ae")]
    indexer_last_log_checksum: Hash,
}

/// Get information about this HOPR Node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/info"),
        description = "Get information about this HOPR Node",
        responses(
            (status = 200, description = "Fetched node version", body = NodeInfoResponse),
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

    let chain_config = hopr.chain_config();
    let safe_config = hopr.get_safe_config();
    let network = hopr.network();
    let me_address = hopr.me_onchain();

    let indexer_state_info = match hopr.get_indexer_state().await {
        Ok(info) => info,
        Err(error) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    };

    let is_eligible = hopr.is_allowed_to_access_network(either::Right(me_address)).await?;

    match hopr.get_channel_closure_notice_period().await {
        Ok(channel_closure_notice_period) => {
            let body = NodeInfoResponse {
                network,
                announced_address: hopr.local_multiaddresses(),
                listening_address: hopr.local_multiaddresses(),
                chain: chain_config.id,
                provider: hopr.get_provider(),
                hopr_token: chain_config.token,
                hopr_channels: chain_config.channels,
                hopr_network_registry: chain_config.network_registry,
                hopr_node_safe_registry: chain_config.node_safe_registry,
                hopr_management_module: chain_config.module_implementation,
                hopr_node_safe: safe_config.safe_address,
                is_eligible,
                connectivity_status: hopr.network_health().await,
                channel_closure_period: channel_closure_notice_period.as_secs(),
                indexer_block: indexer_state_info.latest_block_number,
                indexer_last_log_block: indexer_state_info.latest_log_block_number,
                indexer_last_log_checksum: indexer_state_info.latest_log_checksum,
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
            for (peer_id, address, mas) in nodes.into_iter() {
                body.insert(
                    address.to_string(),
                    EntryNode {
                        multiaddrs: mas,
                        is_eligible: hopr.is_allowed_to_access_network(either::Left(&peer_id)).await?,
                    },
                );
            }

            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(error) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    }
}
