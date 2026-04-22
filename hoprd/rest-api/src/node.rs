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
        node::{HasChainApi, HasNetworkView},
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
                channel_closure_period: channel_closure_notice_period.as_secs(),
            };

            Ok((StatusCode::OK, Json(body)).into_response())
        }
        Err(error) => Ok((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(error)).into_response()),
    }
}

#[cfg(test)]
mod tests {
    use axum::{Router, body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    use super::*;
    use crate::testing::StubUnit;

    fn node_router() -> Router {
        let state: Arc<InternalState<StubUnit>> = Arc::new(InternalState {
            hoprd_cfg: serde_json::json!({
                "network": "test-network",
                "provider": "http://localhost:8545"
            }),
            auth: Arc::new(crate::config::Auth::None),
            hopr: Arc::new(StubUnit),
            open_listeners: Arc::new(hopr_utils_session::ListenerJoinHandles::default()),
            default_listen_host: "127.0.0.1:0".parse().unwrap(),
        });
        Router::new()
            .route(&format!("{BASE_PATH}/node/version"), get(version))
            .route(
                &format!("{BASE_PATH}/node/configuration"),
                get(configuration::<StubUnit>),
            )
            .with_state(state)
    }

    #[tokio::test]
    async fn version_should_return_app_version() -> anyhow::Result<()> {
        let app = node_router();
        let resp = app
            .oneshot(Request::get(&format!("{BASE_PATH}/node/version")).body(Body::empty())?)
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
            .oneshot(Request::get(&format!("{BASE_PATH}/node/configuration")).body(Body::empty())?)
            .await?;
        assert_eq!(resp.status(), StatusCode::OK);

        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await?;
        let body: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(body["network"], "test-network");
        assert_eq!(body["provider"], "http://localhost:8545");
        Ok(())
    }
}
