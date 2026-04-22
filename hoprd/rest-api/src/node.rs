// HashMap is used inside the utoipa macro attribute on the `configuration` endpoint.
#[allow(unused_imports)]
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::{
    Address, Multiaddr,
    api::{
        network::{Health, NetworkView},
        node::{ComponentStatus, HasChainApi, HasNetworkView, IncentiveChannelOperations},
    },
};
use serde::Serialize;
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
        "chainStatus": "Ready",
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
    /// Chain/blokli connector status.
    #[schema(value_type = String, example = "Ready")]
    chain_status: String,
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

    let identity = hopr.identity();

    let provider_url = state
        .hoprd_cfg
        .as_object()
        .and_then(|cfg| cfg.get("blokli_url"))
        .and_then(|v| v.as_str());

    match futures::try_join!(hopr.chain_info(), hopr.get_channel_closure_notice_period()) {
        Ok((info, channel_closure_notice_period)) => {
            let listening: Vec<Multiaddr> = hopr.network_view().listening_as().into_iter().collect();
            let body = NodeInfoResponse {
                announced_address: listening.clone(),
                listening_address: listening,
                provider_url: provider_url.unwrap_or("n/a").to_owned(),
                hopr_network_name: info.hopr_network_name,
                hopr_node_safe: identity.safe_address,
                connectivity_status: hopr.network_view().health(),
                chain_status: HasChainApi::status(&*hopr).to_string(),
                channel_closure_period: channel_closure_notice_period.as_secs(),
            };

            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(error) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    }
}

// ---------------------------------------------------------------------------
// Node status endpoint
// ---------------------------------------------------------------------------

fn component_status_to_info(status: &ComponentStatus) -> ComponentStatusInfo {
    match status {
        ComponentStatus::Ready => ComponentStatusInfo {
            status: "Ready".into(),
            detail: None,
        },
        ComponentStatus::Initializing(d) => ComponentStatusInfo {
            status: "Initializing".into(),
            detail: Some(d.to_string()),
        },
        ComponentStatus::Degraded(d) => ComponentStatusInfo {
            status: "Degraded".into(),
            detail: Some(d.to_string()),
        },
        ComponentStatus::Unavailable(d) => ComponentStatusInfo {
            status: "Unavailable".into(),
            detail: Some(d.to_string()),
        },
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "overall": "Ready",
    "nodeState": "Node is running",
    "components": {
        "chain": { "status": "Ready" },
        "network": { "status": "Ready" },
        "transport": { "status": "Ready" }
    }
}))]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeStatusResponse {
    /// Aggregated status across all components.
    #[schema(example = "Ready")]
    overall: String,
    /// Current node lifecycle state.
    #[schema(example = "Node is running")]
    node_state: String,
    /// Per-component status breakdown.
    components: ComponentStatusesResponse,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentStatusesResponse {
    chain: ComponentStatusInfo,
    network: ComponentStatusInfo,
    transport: ComponentStatusInfo,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ComponentStatusInfo {
    #[schema(example = "Ready")]
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

/// Get the aggregated status of this HOPR node and its individual components.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/node/status"),
    description = "Get the aggregated status of this HOPR node and its individual components",
    responses(
        (status = 200, description = "Fetched node status", body = NodeStatusResponse),
        (status = 401, description = "Invalid authorization token.", body = ApiError),
    ),
    security(
        ("api_token" = []),
        ("bearer_token" = [])
    ),
    tag = "Node"
)]
pub(super) async fn status(State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    let hopr = &state.hopr;
    let statuses = hopr.component_statuses();
    let overall = statuses.aggregate();

    let body = NodeStatusResponse {
        overall: overall.to_string(),
        node_state: statuses.node_state.to_string(),
        components: ComponentStatusesResponse {
            chain: component_status_to_info(&statuses.chain),
            network: component_status_to_info(&statuses.network),
            transport: component_status_to_info(&statuses.transport),
        },
    };

    (StatusCode::OK, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn component_status_to_info_ready() {
        let info = component_status_to_info(&ComponentStatus::Ready);
        assert_eq!(info.status, "Ready");
        assert!(info.detail.is_none());
    }

    #[test]
    fn component_status_to_info_degraded() {
        let info = component_status_to_info(&ComponentStatus::Degraded(Cow::Borrowed("low peers")));
        assert_eq!(info.status, "Degraded");
        assert_eq!(info.detail.as_deref(), Some("low peers"));
    }

    #[test]
    fn component_status_to_info_unavailable() {
        let info = component_status_to_info(&ComponentStatus::Unavailable("down".into()));
        assert_eq!(info.status, "Unavailable");
        assert_eq!(info.detail.as_deref(), Some("down"));
    }

    #[test]
    fn component_status_to_info_initializing() {
        let info = component_status_to_info(&ComponentStatus::Initializing(Cow::Borrowed("starting")));
        assert_eq!(info.status, "Initializing");
        assert_eq!(info.detail.as_deref(), Some("starting"));
    }
}
