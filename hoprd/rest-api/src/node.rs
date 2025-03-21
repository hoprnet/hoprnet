use axum::{
    extract::{Json, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use hopr_crypto_types::prelude::Hash;
use hopr_lib::{Address, AsUnixTimestamp, GraphExportConfig, Health, Multiaddr};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{collections::HashMap, sync::Arc};

use crate::{
    checksum_address_serializer, option_checksum_address_serializer, ApiError, ApiErrorStatus, InternalState, BASE_PATH,
};

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "version": "2.1.0",
        "apiVersion": "3.10.0"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeVersionResponse {
    version: String,
    api_version: String,
}

/// Get the release version of the running node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/version"),
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
        responses(
            (status = 200, description = "Fetched node configuration", body = String),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Configuration"
    )]
pub(super) async fn configuration(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    (StatusCode::OK, state.hoprd_cfg.clone()).into_response()
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
#[schema(example = json!({
        "quality": 0.7
    }))]
#[into_params(parameter_in = Query)]
pub(crate) struct NodePeersQueryRequest {
    #[schema(required = false)]
    #[serde(default)]
    quality: f64,
}

#[derive(Debug, Default, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "sent": 10,
    "success": 10
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct HeartbeatInfo {
    sent: u64,
    success: u64,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PeerInfo {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    peer_id: PeerId,
    #[serde(serialize_with = "option_checksum_address_serializer")]
    #[schema(value_type = Option<String>)]
    peer_address: Option<Address>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = Option<String>)]
    multiaddr: Option<Multiaddr>,
    heartbeats: HeartbeatInfo,
    last_seen: u128,
    last_seen_latency: u128,
    quality: f64,
    backoff: f64,
    is_new: bool,
    reported_version: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "peerId": "12D3KooWRWeaTozREYHzWTbuCYskdYhED1MXpDwTrmccwzFrd2mEA",
    "peerAddress": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
    "multiaddr": "/ip4/178.12.1.9/tcp/19092"
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct AnnouncedPeer {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    peer_id: PeerId,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    peer_address: Address,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = Option<String>)]
    multiaddr: Option<Multiaddr>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodePeersResponse {
    connected: Vec<PeerInfo>,
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

                Some((address, peer_id, multiaddresses, info))
            }
        })
        .map(|(address, peer_id, mas, info)| PeerInfo {
            peer_id,
            peer_address: address,
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
                peer_id: announced.public_key.into(),
                peer_address: announced.chain_addr,
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
use hopr_metrics::metrics::gather_all_metrics as collect_hopr_metrics;

#[cfg(any(not(feature = "prometheus"), test))]
fn collect_hopr_metrics() -> Result<String, ApiErrorStatus> {
    Err(ApiErrorStatus::UnknownFailure("BUILT WITHOUT METRICS SUPPORT".into()))
}

/// Retrieve Prometheus metrics from the running node.
#[utoipa::path(
        get,
        request_body(content_type = "text/plain"),
        path = const_format::formatcp!("{BASE_PATH}/node/metrics"),
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
        Err(error) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response(),
    }
}

#[derive(Debug, Clone, Deserialize, Default, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
#[serde(default, rename_all = "camelCase")]
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
    /// Export the entire graph in raw JSON format, that can be later
    /// used to load the graph into e.g. a unit test.
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
        }
    }
}

/// Retrieve node's channel graph in DOT or JSON format.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/node/graph"),
    params(GraphExportQuery),
    responses(
            (status = 200, description = "Fetched channel graph", body = String),
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
            Ok(raw_graph) => (StatusCode::OK, raw_graph).into_response(),
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
        "hoprNodeSageRegistry": "0x0dcd1bf9a1b36ce34237eeafef220932846bcd82",
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
pub(crate) struct NodeInfoResponse {
    network: String,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>)]
    announced_address: Vec<Multiaddr>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>)]
    listening_address: Vec<Multiaddr>,
    chain: String,
    provider: String,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    hopr_token: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    hopr_channels: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    hopr_network_registry: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    hopr_node_safe_registry: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    hopr_management_module: Address,
    #[serde(serialize_with = "checksum_address_serializer")]
    #[schema(value_type = String)]
    hopr_node_safe: Address,
    is_eligible: bool,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    connectivity_status: Health,
    /// Channel closure period in seconds
    channel_closure_period: u64,
    indexer_block: u32,
    indexer_last_log_block: u32,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    indexer_last_log_checksum: Hash,
}

/// Get information about this HOPR Node.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/info"),
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

    let indexer_state_info = match hopr.get_indexer_state().await {
        Ok(info) => info,
        Err(error) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    };

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
                is_eligible: hopr.is_allowed_to_access_network(&hopr.me_peer_id()).await?,
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
pub(crate) struct EntryNode {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    #[schema(value_type = Vec<String>)]
    multiaddrs: Vec<Multiaddr>,
    is_eligible: bool,
}

/// List all known entry nodes with multiaddrs and eligibility.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/entryNodes"),
        responses(
            (status = 200, description = "Fetched public nodes' information", body = HashMap<String, EntryNode>, example = json!({
                "0x188c4462b75e46f0c7262d7f48d182447b93a93c": {
                    "isElligible": true,
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
                        is_eligible: hopr.is_allowed_to_access_network(&peer_id).await?,
                    },
                );
            }

            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(error) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    }
}
