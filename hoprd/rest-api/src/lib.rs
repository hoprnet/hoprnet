//! REST API for the HOPRd node.
pub mod config;

mod account;
mod alias;
mod channels;
mod checks;
mod messages;
mod network;
mod node;
mod peers;
mod preconditions;
mod prometheus;
mod session;
mod tickets;
mod types;

pub use session::{HOPR_TCP_BUFFER_SIZE, HOPR_UDP_BUFFER_SIZE, HOPR_UDP_QUEUE_SIZE};

use async_lock::RwLock;
use axum::{
    extract::Json,
    http::{header::AUTHORIZATION, status::StatusCode, Method},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::iter::once;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::TraceLayer,
    validate_request::ValidateRequestHeaderLayer,
};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_scalar::{Scalar, Servable};

use crate::config::Auth;
use hopr_lib::{errors::HoprLibError, Hopr};
use hopr_network_types::prelude::IpProtocol;

pub(crate) const BASE_PATH: &str = "/api/v3";

#[derive(Clone)]
pub(crate) struct AppState {
    pub hopr: Arc<Hopr>, // checks
}

pub type MessageEncoder = fn(&[u8]) -> Box<[u8]>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ListenerId(pub IpProtocol, pub std::net::SocketAddr);

pub type ListenerJoinHandles = Arc<RwLock<HashMap<ListenerId, (String, hopr_async_runtime::prelude::JoinHandle<()>)>>>;

#[derive(Clone)]
pub(crate) struct InternalState {
    pub hoprd_cfg: String,
    pub auth: Arc<Auth>,
    pub hopr: Arc<Hopr>,
    pub inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    pub hoprd_db: Arc<hoprd_db_api::db::HoprdDb>,
    pub msg_encoder: Option<MessageEncoder>,
    pub open_listeners: ListenerJoinHandles,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        account::addresses,
        account::balances,
        account::withdraw,
        alias::aliases,
        alias::set_alias,
        alias::get_alias,
        alias::delete_alias,
        alias::clear_aliases,
        channels::close_channel,
        channels::fund_channel,
        channels::list_channels,
        channels::open_channel,
        channels::show_channel,
        checks::eligiblez,
        checks::healthyz,
        checks::readyz,
        checks::startedz,
        messages::delete_messages,
        messages::peek,
        messages::peek_all,
        messages::pop,
        messages::pop_all,
        messages::send_message,
        messages::size,
        network::price,
        network::probability,
        node::configuration,
        node::entry_nodes,
        node::info,
        node::metrics,
        node::peers,
        node::version,
        peers::ping_peer,
        peers::show_peer_info,
        session::create_client,
        session::list_clients,
        session::close_client,
        tickets::aggregate_tickets_in_channel,
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
            alias::PeerIdResponse, alias::AliasDestinationBodyRequest,
            channels::ChannelsQueryRequest,channels::CloseChannelResponse, channels::OpenChannelBodyRequest, channels::OpenChannelResponse,
            channels::NodeChannel, channels::NodeChannelsResponse, channels::ChannelInfoResponse, channels::FundBodyRequest,
            messages::MessagePopAllResponse,
            messages::MessagePopResponse, messages::SendMessageResponse, messages::SendMessageBodyRequest, messages::SizeResponse, messages::TagQueryRequest, messages::GetMessageBodyRequest,
            network::TicketPriceResponse,
            network::TicketProbabilityResponse,
            node::EntryNode, node::NodeInfoResponse, node::NodePeersQueryRequest,
            node::HeartbeatInfo, node::PeerInfo, node::AnnouncedPeer, node::NodePeersResponse, node::NodeVersionResponse,
            peers::NodePeerInfoResponse, peers::PingResponse,
            session::SessionClientRequest, session::SessionClientResponse, session::SessionCloseClientRequest,
            tickets::NodeTicketStatisticsResponse, tickets::ChannelTicket,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Account", description = "HOPR node account endpoints"),
        (name = "Alias", description = "HOPR node internal non-persistent alias endpoints"),
        (name = "Channels", description = "HOPR node chain channels manipulation endpoints"),
        (name = "Checks", description = "HOPR node functionality checks"),
        (name = "Messages", description = "HOPR node message manipulation endpoints"),
        (name = "Node", description = "HOPR node information endpoints"),
        (name = "Peers", description = "HOPR node peer manipulation endpoints"),
        (name = "Tickets", description = "HOPR node ticket management endpoints"),
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
                    .build(),
            ),
        );
        components.add_security_scheme(
            "api_token",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Auth-Token"))),
        );
    }
}

/// Parameters needed to construct the Rest API via [`serve_api`].
pub struct RestApiParameters {
    pub listener: TcpListener,
    pub hoprd_cfg: String,
    pub cfg: crate::config::Api,
    pub hopr: Arc<hopr_lib::Hopr>,
    pub hoprd_db: Arc<hoprd_db_api::db::HoprdDb>,
    pub inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    pub session_listener_sockets: ListenerJoinHandles,
    pub msg_encoder: Option<MessageEncoder>,
}

/// Starts the Rest API listener and router.
pub async fn serve_api(params: RestApiParameters) -> Result<(), std::io::Error> {
    let RestApiParameters {
        listener,
        hoprd_cfg,
        cfg,
        hopr,
        hoprd_db,
        inbox,
        session_listener_sockets,
        msg_encoder,
    } = params;

    let router = build_api(
        hoprd_cfg,
        cfg,
        hopr,
        inbox,
        hoprd_db,
        session_listener_sockets,
        msg_encoder,
    )
    .await;
    axum::serve(listener, router).await
}

#[allow(clippy::too_many_arguments)]
async fn build_api(
    hoprd_cfg: String,
    cfg: crate::config::Api,
    hopr: Arc<hopr_lib::Hopr>,
    inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    hoprd_db: Arc<hoprd_db_api::db::HoprdDb>,
    open_listeners: ListenerJoinHandles,
    msg_encoder: Option<MessageEncoder>,
) -> Router {
    let state = AppState { hopr };
    let inner_state = InternalState {
        auth: Arc::new(cfg.auth.clone()),
        hoprd_cfg,
        hopr: state.hopr.clone(),
        msg_encoder,
        inbox,
        hoprd_db,
        open_listeners,
    };

    Router::new()
        .nest("/", Router::new().merge(Scalar::with_url("/scalar", ApiDoc::openapi())))
        .nest(
            "/",
            Router::new()
                .route("/startedz", get(checks::startedz))
                .route("/readyz", get(checks::readyz))
                .route("/healthyz", get(checks::healthyz))
                .route("/eligiblez", get(checks::eligiblez))
                .with_state(state.into()),
        )
        .nest(
            BASE_PATH,
            Router::new()
                .route("/aliases", get(alias::aliases))
                .route("/aliases", post(alias::set_alias))
                .route("/aliases", delete(alias::clear_aliases))
                .route("/aliases/:alias", get(alias::get_alias))
                .route("/aliases/:alias", delete(alias::delete_alias))
                .route("/account/addresses", get(account::addresses))
                .route("/account/balances", get(account::balances))
                .route("/account/withdraw", post(account::withdraw))
                .route("/peers/:destination", get(peers::show_peer_info))
                .route("/channels", get(channels::list_channels))
                .route("/channels", post(channels::open_channel))
                .route("/channels/:channelId", get(channels::show_channel))
                .route("/channels/:channelId/tickets", get(tickets::show_channel_tickets))
                .route("/channels/:channelId", delete(channels::close_channel))
                .route("/channels/:channelId/fund", post(channels::fund_channel))
                .route(
                    "/channels/:channelId/tickets/redeem",
                    post(tickets::redeem_tickets_in_channel),
                )
                .route(
                    "/channels/:channelId/tickets/aggregate",
                    post(tickets::aggregate_tickets_in_channel),
                )
                .route("/tickets", get(tickets::show_all_tickets))
                .route("/tickets/redeem", post(tickets::redeem_all_tickets))
                .route("/tickets/statistics", get(tickets::show_ticket_statistics))
                .route("/tickets/statistics", delete(tickets::reset_ticket_statistics))
                .route("/messages", delete(messages::delete_messages))
                .route("/messages", post(messages::send_message))
                .route("/messages/pop", post(messages::pop))
                .route("/messages/pop-all", post(messages::pop_all))
                .route("/messages/peek", post(messages::peek))
                .route("/messages/peek-all", post(messages::peek_all))
                .route("/messages/size", get(messages::size))
                .route("/network/price", get(network::price))
                .route("/network/probability", get(network::probability))
                .route("/node/version", get(node::version))
                .route("/node/configuration", get(node::configuration))
                .route("/node/info", get(node::info))
                .route("/node/peers", get(node::peers))
                .route("/node/entryNodes", get(node::entry_nodes))
                .route("/node/metrics", get(node::metrics))
                .route("/peers/:destination/ping", post(peers::ping_peer))
                .route("/session/websocket", get(session::websocket))
                .route("/session/:protocol", post(session::create_client))
                .route("/session/:protocol", get(session::list_clients))
                .route("/session/:protocol", delete(session::close_client))
                .with_state(inner_state.clone().into())
                .layer(middleware::from_fn_with_state(inner_state, preconditions::authenticate)),
        )
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
                .layer(middleware::from_fn(prometheus::record))
                .layer(CompressionLayer::new())
                .layer(ValidateRequestHeaderLayer::accept("application/json"))
                .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION))),
        )
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[schema(example = json!({
    "status": "INVALID_INPUT",
    "error": "Invalid value passed in parameter 'XYZ'"
}))]
pub(crate) struct ApiError {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Enumerates all API request errors
/// Note that `ApiError` should not be instantiated directly, but always rather through the `ApiErrorStatus`.
#[derive(Debug, Clone, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
enum ApiErrorStatus {
    InvalidInput,
    /// An invalid application tag from the reserved range was provided.
    InvalidApplicationTag,
    InvalidChannelId,
    PeerNotFound,
    ChannelNotFound,
    TicketsNotFound,
    NotEnoughBalance,
    NotEnoughAllowance,
    ChannelAlreadyOpen,
    ChannelNotOpen,
    AliasNotFound,
    DatabaseError,
    UnsupportedFeature,
    Timeout,
    Unauthorized,
    InvalidQuality,
    NotReady,
    #[strum(serialize = "INVALID_PATH")]
    InvalidPath(String),
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
        (StatusCode::INTERNAL_SERVER_ERROR, self).into_response()
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
