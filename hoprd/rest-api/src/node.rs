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
    Address, IncentiveChannelOperations, Multiaddr,
    api::{
        network::{Health, NetworkView},
        node::{ComponentStatus, HasChainApi, HasNetworkView},
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
pub(super) async fn configuration<H: Send + Sync + 'static>(
    State(state): State<Arc<InternalState<H>>>,
) -> impl IntoResponse {
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
pub(super) async fn info<
    H: HasChainApi<ChainError = hopr_lib::errors::HoprLibError> + HasNetworkView + Send + Sync + 'static,
>(
    State(state): State<Arc<InternalState<H>>>,
) -> Result<impl IntoResponse, ApiError> {
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
pub(super) async fn status<
    H: hopr_lib::api::node::HoprNodeOperations
        + HasChainApi<ChainError = hopr_lib::errors::HoprLibError>
        + HasNetworkView
        + hopr_lib::api::node::HasTransportApi
        + Send
        + Sync
        + 'static,
>(
    State(state): State<Arc<InternalState<H>>>,
) -> impl IntoResponse {
    use hopr_lib::api::node::{HasTransportApi, HoprNodeOperations};

    let hopr = &state.hopr;

    let chain = HasChainApi::status(&**hopr);
    let network = HasNetworkView::status(&**hopr);
    let transport = HasTransportApi::status(&**hopr);
    let node_state = HoprNodeOperations::status(&**hopr);

    let statuses = [&chain, &network, &transport];
    let overall = if statuses.iter().any(|s| s.is_unavailable()) {
        ComponentStatus::Unavailable("one or more components unavailable".into())
    } else if statuses.iter().any(|s| s.is_degraded()) {
        ComponentStatus::Degraded("one or more components degraded".into())
    } else if statuses.iter().any(|s| s.is_initializing()) {
        ComponentStatus::Initializing("one or more components initializing".into())
    } else {
        ComponentStatus::Ready
    };

    let body = NodeStatusResponse {
        overall: overall.to_string(),
        node_state: node_state.to_string(),
        components: ComponentStatusesResponse {
            chain: component_status_to_info(&chain),
            network: component_status_to_info(&network),
            transport: component_status_to_info(&transport),
        },
    };

    (StatusCode::OK, Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    use super::*;
    use crate::testing::NoopNode;

    fn node_router() -> Router {
        let state: Arc<InternalState<NoopNode>> = Arc::new(InternalState {
            hoprd_cfg: serde_json::json!({
                "network": "test-network",
                "provider": "http://localhost:8545"
            }),
            auth: Arc::new(crate::config::Auth::None),
            hopr: Arc::new(NoopNode),
            open_listeners: Arc::new(hopr_utils_session::ListenerJoinHandles::default()),
            default_listen_host: "127.0.0.1:0".parse().unwrap(),
        });
        Router::new()
            .route(&format!("{BASE_PATH}/node/version"), get(version))
            .route(
                &format!("{BASE_PATH}/node/configuration"),
                get(configuration::<NoopNode>),
            )
            .with_state(state)
    }

    #[tokio::test]
    async fn version_should_return_app_version() -> anyhow::Result<()> {
        let app = node_router();
        let resp = app
            .oneshot(Request::get(format!("{BASE_PATH}/node/version")).body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await?;
        let body: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert!(body["version"].as_str().is_some());
        Ok(())
    }

    #[tokio::test]
    async fn configuration_should_return_hoprd_config() -> anyhow::Result<()> {
        let app = node_router();
        let resp = app
            .oneshot(Request::get(format!("{BASE_PATH}/node/configuration")).body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await?;
        let body: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(body["network"], "test-network");
        assert_eq!(body["provider"], "http://localhost:8545");
        Ok(())
    }

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

    #[test]
    fn component_status_to_info_with_owned_cow() {
        let info = component_status_to_info(&ComponentStatus::Degraded(Cow::Owned("dynamic detail".to_string())));
        assert_eq!(info.status, "Degraded");
        assert_eq!(info.detail.as_deref(), Some("dynamic detail"));
    }

    #[test]
    fn component_status_to_info_empty_detail() {
        let info = component_status_to_info(&ComponentStatus::Degraded(Cow::Borrowed("")));
        assert_eq!(info.detail.as_deref(), Some(""));
    }

    #[test]
    fn node_status_response_serializes_correctly() {
        let body = NodeStatusResponse {
            overall: "Ready".into(),
            node_state: "Node is running".into(),
            components: ComponentStatusesResponse {
                chain: ComponentStatusInfo {
                    status: "Ready".into(),
                    detail: None,
                },
                network: ComponentStatusInfo {
                    status: "Degraded".into(),
                    detail: Some("low peers".into()),
                },
                transport: ComponentStatusInfo {
                    status: "Ready".into(),
                    detail: None,
                },
            },
        };
        let json = serde_json::to_value(&body).unwrap();
        assert_eq!(json["overall"], "Ready");
        assert_eq!(json["nodeState"], "Node is running");
        assert_eq!(json["components"]["chain"]["status"], "Ready");
        assert!(json["components"]["chain"]["detail"].is_null());
        assert_eq!(json["components"]["network"]["detail"], "low peers");
    }

    #[test]
    fn component_status_info_skips_none_detail_in_json() {
        let info = ComponentStatusInfo {
            status: "Ready".into(),
            detail: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(!json.contains("detail"), "None detail should be skipped");
    }

    #[test]
    fn component_status_info_includes_some_detail_in_json() {
        let info = ComponentStatusInfo {
            status: "Degraded".into(),
            detail: Some("reason".into()),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"detail\":\"reason\""));
    }
}
