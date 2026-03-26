use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    Address, Multiaddr,
    api::{network::Health, node::HoprNodeNetworkOperations},
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
