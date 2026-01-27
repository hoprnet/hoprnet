//! REST API for the HOPRd node.
pub mod config;

mod account;
mod channels;
mod checks;
mod middleware;
mod network;
mod node;
mod peers;
mod root;
mod session;
mod tickets;

pub(crate) mod env {
    /// Name of the environment variable specifying automatic port range selection for Sessions.
    /// Expected format: "<start_port>:<end_port>" (e.g., "9091:9099")
    pub const HOPRD_SESSION_PORT_RANGE: &str = "HOPRD_SESSION_PORT_RANGE";
}

use std::{
    error::Error,
    iter::once,
    sync::{Arc, atomic::AtomicU16},
};

use axum::{
    Router,
    extract::Json,
    http::{Method, header::AUTHORIZATION, status::StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use hopr_chain_connector::HoprBlockchainSafeConnector;
use hopr_db_node::HoprNodeDb;
use hopr_lib::{Address, Hopr, errors::HoprLibError};
use hopr_utils_session::ListenerJoinHandles;
use serde::Serialize;
pub use session::{HOPR_TCP_BUFFER_SIZE, HOPR_UDP_BUFFER_SIZE, HOPR_UDP_QUEUE_SIZE};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::TraceLayer,
    validate_request::ValidateRequestHeaderLayer,
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

use crate::config::Auth;

pub(crate) const BASE_PATH: &str = const_format::formatcp!("/api/v{}", env!("CARGO_PKG_VERSION_MAJOR"));

#[cfg(not(feature = "test-fixtures"))]
type HoprBlokliConnector = HoprBlockchainSafeConnector<hopr_chain_connector::blokli_client::BlokliClient>;

#[cfg(feature = "test-fixtures")]
type HoprBlokliConnector = HoprBlockchainSafeConnector<
    hopr_chain_connector::testing::BlokliTestClient<hopr_chain_connector::testing::FullStateEmulator>,
>;

#[derive(Clone)]
pub(crate) struct AppState {
    pub hopr: Arc<Hopr<Arc<HoprBlokliConnector>, HoprNodeDb>>, // checks
}

pub type MessageEncoder = fn(&[u8]) -> Box<[u8]>;

#[derive(Clone)]
pub(crate) struct InternalState {
    pub hoprd_cfg: serde_json::Value,
    pub auth: Arc<Auth>,
    pub hopr: Arc<Hopr<Arc<HoprBlokliConnector>, HoprNodeDb>>,
    pub websocket_active_count: Arc<AtomicU16>,
    pub open_listeners: Arc<ListenerJoinHandles>,
    pub default_listen_host: std::net::SocketAddr,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        account::addresses,
        account::balances,
        account::withdraw,
        channels::close_channel,
        channels::fund_channel,
        channels::list_channels,
        channels::open_channel,
        channels::show_channel,
        checks::eligiblez,
        checks::healthyz,
        checks::readyz,
        checks::startedz,
        network::price,
        network::probability,
        node::configuration,
        node::entry_nodes,
        node::info,
        node::peers,
        node::version,
        peers::ping_peer,
        peers::show_peer_info,
        root::metrics,
        session::create_client,
        session::list_clients,
        session::adjust_session,
        session::session_config,
        session::session_metrics,
        session::close_client,
        tickets::redeem_all_tickets,
        tickets::redeem_tickets_in_channel,
        tickets::show_all_tickets,
        tickets::show_channel_tickets,
        tickets::show_ticket_statistics,
        tickets::reset_ticket_statistics,
    ),
    components(
        schemas(
            ApiError,
            account::AccountAddressesResponse, account::AccountBalancesResponse, account::WithdrawBodyRequest, account::WithdrawResponse,
            channels::ChannelsQueryRequest,channels::CloseChannelResponse, channels::OpenChannelBodyRequest, channels::OpenChannelResponse, channels::FundChannelResponse,
            channels::NodeChannel, channels::NodeChannelsResponse, channels::ChannelInfoResponse, channels::FundBodyRequest,
            network::TicketPriceResponse,
            network::TicketProbabilityResponse,
            node::EntryNode, node::NodeInfoResponse, node::NodePeersQueryRequest,
            node::HeartbeatInfo, node::PeerObservations, node::AnnouncedPeer, node::NodePeersResponse, node::NodeVersionResponse,
            peers::NodePeerInfoResponse, peers::PingResponse,
            session::SessionClientRequest, session::SessionCapability, session::RoutingOptions, session::SessionTargetSpec, session::SessionClientResponse, session::IpProtocol, session::SessionConfig,
            session::SessionMetricsResponse, session::SessionMetricsLifetime, session::SessionMetricsFrameBuffer, session::SessionMetricsAck, session::SessionMetricsSurb, session::SessionMetricsTransport, session::SessionMetricsState, session::SessionMetricsAckMode,
            tickets::NodeTicketStatisticsResponse, tickets::ChannelTicket,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Account", description = "HOPR node account endpoints"),
        (name = "Channels", description = "HOPR node chain channels manipulation endpoints"),
        (name = "Configuration", description = "HOPR node configuration endpoints"),
        (name = "Checks", description = "HOPR node functionality checks"),
        (name = "Network", description = "HOPR node network endpoints"),
        (name = "Node", description = "HOPR node information endpoints"),
        (name = "Peers", description = "HOPR node peer manipulation endpoints"),
        (name = "Session", description = "HOPR node session management endpoints"),
        (name = "Tickets", description = "HOPR node ticket management endpoints"),
        (name = "Metrics", description = "HOPR node metrics endpoints"),
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .as_mut()
            .expect("components should be registered at this point");

        components.add_security_scheme(
            "bearer_token",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("token")
                    .description(Some("Bearer token authentication".to_string()))
                    .build(),
            ),
        );
        components.add_security_scheme(
            "api_token",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::with_description(
                "X-Auth-Token",
                "API Token",
            ))),
        );
    }
}

/// Parameters needed to construct the Rest API via [`serve_api`].
pub struct RestApiParameters {
    pub listener: TcpListener,
    pub hoprd_cfg: serde_json::Value,
    pub cfg: crate::config::Api,
    pub hopr: Arc<Hopr<Arc<HoprBlokliConnector>, HoprNodeDb>>,
    pub session_listener_sockets: Arc<ListenerJoinHandles>,
    pub default_session_listen_host: std::net::SocketAddr,
}

/// Starts the Rest API listener and router.
pub async fn serve_api(params: RestApiParameters) -> Result<(), std::io::Error> {
    let RestApiParameters {
        listener,
        hoprd_cfg,
        cfg,
        hopr,
        session_listener_sockets,
        default_session_listen_host,
    } = params;

    let router = build_api(
        hoprd_cfg,
        cfg,
        hopr,
        session_listener_sockets,
        default_session_listen_host,
    )
    .await;
    axum::serve(listener, router).await
}

#[allow(clippy::too_many_arguments)]
async fn build_api(
    hoprd_cfg: serde_json::Value,
    cfg: crate::config::Api,
    hopr: Arc<Hopr<Arc<HoprBlokliConnector>, HoprNodeDb>>,
    open_listeners: Arc<ListenerJoinHandles>,
    default_listen_host: std::net::SocketAddr,
) -> Router {
    let state = AppState { hopr };
    let inner_state = InternalState {
        auth: Arc::new(cfg.auth.clone()),
        hoprd_cfg,
        hopr: state.hopr.clone(),
        open_listeners,
        default_listen_host,
        websocket_active_count: Arc::new(AtomicU16::new(0)),
    };

    Router::new()
        .merge(
            Router::new()
                .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
                .merge(Scalar::with_url("/scalar", ApiDoc::openapi())),
        )
        .merge(
            Router::new()
                .route("/startedz", get(checks::startedz))
                .route("/readyz", get(checks::readyz))
                .route("/healthyz", get(checks::healthyz))
                .route("/eligiblez", get(checks::eligiblez))
                .layer(
                    ServiceBuilder::new().layer(
                        CorsLayer::new()
                            .allow_methods([Method::GET])
                            .allow_origin(Any)
                            .allow_headers(Any)
                            .max_age(std::time::Duration::from_secs(86400)),
                    ),
                )
                .with_state(state.into()),
        )
        .merge(
            Router::new()
                .route("/metrics", get(root::metrics))
                .layer(axum::middleware::from_fn_with_state(
                    inner_state.clone(),
                    middleware::preconditions::authenticate,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    inner_state.clone(),
                    middleware::preconditions::cap_websockets,
                ))
                .layer(
                    ServiceBuilder::new()
                        .layer(TraceLayer::new_for_http())
                        .layer(
                            CorsLayer::new()
                                .allow_methods([Method::GET])
                                .allow_origin(Any)
                                .allow_headers(Any)
                                .max_age(std::time::Duration::from_secs(86400)),
                        )
                        .layer(axum::middleware::from_fn(middleware::prometheus::record))
                        .layer(CompressionLayer::new())
                        .layer(ValidateRequestHeaderLayer::accept("text/plain"))
                        .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION))),
                ),
        )
        .nest(
            BASE_PATH,
            Router::new()
                .route("/account/addresses", get(account::addresses))
                .route("/account/balances", get(account::balances))
                .route("/account/withdraw", post(account::withdraw))
                .route("/peers/{destination}", get(peers::show_peer_info))
                .route("/channels", get(channels::list_channels))
                .route("/channels", post(channels::open_channel))
                .route("/channels/{channelId}", get(channels::show_channel))
                .route("/channels/{channelId}/tickets", get(tickets::show_channel_tickets))
                .route("/channels/{channelId}", delete(channels::close_channel))
                .route("/channels/{channelId}/fund", post(channels::fund_channel))
                .route(
                    "/channels/{channelId}/tickets/redeem",
                    post(tickets::redeem_tickets_in_channel),
                )
                .route("/tickets", get(tickets::show_all_tickets))
                .route("/tickets/redeem", post(tickets::redeem_all_tickets))
                .route("/tickets/statistics", get(tickets::show_ticket_statistics))
                .route("/tickets/statistics", delete(tickets::reset_ticket_statistics))
                .route("/network/price", get(network::price))
                .route("/network/probability", get(network::probability))
                .route("/node/version", get(node::version))
                .route("/node/configuration", get(node::configuration))
                .route("/node/info", get(node::info))
                .route("/node/peers", get(node::peers))
                .route("/node/entry-nodes", get(node::entry_nodes))
                .route("/peers/{destination}/ping", post(peers::ping_peer))
                .route("/session/config/{id}", get(session::session_config))
                .route("/session/config/{id}", post(session::adjust_session))
                .route("/session/metrics/{id}", get(session::session_metrics))
                .route("/session/websocket", get(session::websocket))
                .route("/session/{protocol}", post(session::create_client))
                .route("/session/{protocol}", get(session::list_clients))
                .route("/session/{protocol}/{ip}/{port}", delete(session::close_client))
                .with_state(inner_state.clone().into())
                .layer(axum::middleware::from_fn_with_state(
                    inner_state.clone(),
                    middleware::preconditions::authenticate,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    inner_state.clone(),
                    middleware::preconditions::cap_websockets,
                ))
                .layer(
                    ServiceBuilder::new()
                        .layer(TraceLayer::new_for_http())
                        .layer(
                            CorsLayer::new()
                                .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::DELETE])
                                .allow_origin(Any)
                                .allow_headers(Any)
                                .max_age(std::time::Duration::from_secs(86400)),
                        )
                        .layer(axum::middleware::from_fn(middleware::prometheus::record))
                        .layer(CompressionLayer::new())
                        .layer(ValidateRequestHeaderLayer::accept("application/json"))
                        .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION))),
                ),
        )
}

fn checksum_address_serializer<S: serde::Serializer>(a: &Address, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&a.to_checksum())
}

fn option_checksum_address_serializer<S: serde::Serializer>(a: &Option<Address>, s: S) -> Result<S::Ok, S::Error> {
    if let Some(addr) = a {
        s.serialize_some(&addr.to_checksum())
    } else {
        s.serialize_none()
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "status": "INVALID_INPUT",
    "error": "Invalid value passed in parameter 'XYZ'"
}))]
/// Standardized error response for the API
pub(crate) struct ApiError {
    #[schema(example = "INVALID_INPUT")]
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Invalid value passed in parameter 'XYZ'")]
    pub error: Option<String>,
}

/// Enumerates all API request errors
/// Note that `ApiError` should not be instantiated directly, but always rather through the `ApiErrorStatus`.
#[allow(unused)] // TODO: some errors can no longer be propagated to the REST API
#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum ApiErrorStatus {
    InvalidInput,
    InvalidChannelId,
    PeerNotFound,
    ChannelNotFound,
    TicketsNotFound,
    Timeout,
    PingError(String),
    Unauthorized,
    TooManyOpenWebsocketConnections,
    InvalidQuality,
    NotReady,
    ListenHostAlreadyUsed,
    SessionNotFound,
    InvalidSessionId,
    #[strum(serialize = "UNKNOWN_FAILURE")]
    UnknownFailure(String),
}

impl From<ApiErrorStatus> for ApiError {
    fn from(value: ApiErrorStatus) -> Self {
        Self {
            status: value.to_string(),
            error: if let ApiErrorStatus::UnknownFailure(e) = value {
                Some(e)
            } else {
                None
            },
        }
    }
}

impl IntoResponse for ApiErrorStatus {
    fn into_response(self) -> Response {
        Json(ApiError::from(self)).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}

// Errors lead to `UnknownFailure` per default
impl<T: Error> From<T> for ApiErrorStatus {
    fn from(value: T) -> Self {
        Self::UnknownFailure(value.to_string())
    }
}

// Errors lead to `UnknownFailure` per default
impl<T> From<T> for ApiError
where
    T: Error + Into<HoprLibError>,
{
    fn from(value: T) -> Self {
        Self {
            status: ApiErrorStatus::UnknownFailure("unknown error".to_string()).to_string(),
            error: Some(value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{http::StatusCode, response::IntoResponse};

    use super::ApiError;

    #[test]
    fn test_api_error_to_response() {
        let error = ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            error: Some("Invalid value passed in parameter 'XYZ'".to_string()),
        };

        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
