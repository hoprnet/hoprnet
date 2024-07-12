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
mod prometheus;
mod tickets;
mod token_authentication;

use async_lock::RwLock;
use axum::{
    extract::Json,
    http::{status::StatusCode, Method},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use bimap::BiHashMap;
use hyper::header::AUTHORIZATION;
use libp2p_identity::PeerId;
use serde::Serialize;
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

use hopr_lib::{errors::HoprLibError, Hopr, TransportOutput};

use crate::config::Auth;

pub(crate) const BASE_PATH: &str = "/api/v3";

#[derive(Clone)]
pub(crate) struct AppState {
    pub hopr: Arc<Hopr>, // checks
}

pub type MessageEncoder = fn(&[u8]) -> Box<[u8]>;

#[derive(Clone)]
pub(crate) struct InternalState {
    pub hoprd_cfg: String,
    pub auth: Arc<Auth>,
    pub hopr: Arc<Hopr>,
    pub inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    pub aliases: Arc<RwLock<BiHashMap<String, PeerId>>>,
    pub websocket_rx: async_broadcast::InactiveReceiver<TransportOutput>,
    pub msg_encoder: Option<MessageEncoder>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        account::addresses,
        account::balances,
        account::withdraw,
        alias::aliases,
        alias::delete_alias,
        alias::get_alias,
        alias::set_alias,
        channels::close_channel,
        channels::fund_channel,
        channels::list_channels,
        channels::open_channel,
        channels::show_channel,
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
        node::configuration,
        node::entry_nodes,
        node::info,
        node::metrics,
        node::peers,
        node::version,
        peers::ping_peer,
        peers::show_peer_info,
        tickets::aggregate_tickets_in_channel,
        tickets::redeem_all_tickets,
        tickets::redeem_tickets_in_channel,
        tickets::show_all_tickets,
        tickets::show_channel_tickets,
        tickets::show_ticket_statistics
    ),
    components(
        schemas(
            ApiError,
            account::AccountAddressesResponse, account::AccountBalancesResponse, account::WithdrawBodyRequest, account::WithdrawResponse,
            alias::PeerIdResponse, alias::AliasPeerIdBodyRequest,
            channels::ChannelsQueryRequest,channels::CloseChannelResponse, channels::OpenChannelBodyRequest, channels::OpenChannelResponse,
            channels::NodeChannel, channels::NodeChannelsResponse, channels::ChannelInfoResponse, channels::FundBodyRequest,
            messages::MessagePopAllResponse,
            messages::MessagePopResponse, messages::SendMessageResponse, messages::SendMessageBodyRequest, messages::SizeResponse, messages::TagQueryRequest, messages::GetMessageBodyRequest,
            network::TicketPriceResponse,
            node::EntryNode, node::NodeInfoResponse, node::NodePeersQueryRequest,
            node::HeartbeatInfo, node::PeerInfo, node::AnnouncedPeer, node::NodePeersResponse, node::NodeVersionResponse,
            peers::NodePeerInfoResponse, peers::PingResponse,
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
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
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

pub async fn serve_api(
    listener: TcpListener,
    hoprd_cfg: String,
    cfg: crate::config::Api,
    hopr: Arc<hopr_lib::Hopr>,
    inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    websocket_rx: async_broadcast::InactiveReceiver<TransportOutput>,
    msg_encoder: Option<MessageEncoder>,
) -> Result<(), std::io::Error> {
    let router = build_api(hoprd_cfg, cfg, hopr, inbox, websocket_rx, msg_encoder).await;
    axum::serve(listener, router).await
}

async fn build_api(
    hoprd_cfg: String,
    cfg: crate::config::Api,
    hopr: Arc<hopr_lib::Hopr>,
    inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    websocket_rx: async_broadcast::InactiveReceiver<TransportOutput>,
    msg_encoder: Option<MessageEncoder>,
) -> Router {
    // Prepare alias part of the state
    let aliases: Arc<RwLock<BiHashMap<String, PeerId>>> = Arc::new(RwLock::new(BiHashMap::new()));
    aliases.write().await.insert("me".to_owned(), hopr.me_peer_id());

    let state = AppState { hopr };
    let inner_state = InternalState {
        auth: Arc::new(cfg.auth.clone()),
        hoprd_cfg,
        hopr: state.hopr.clone(),
        msg_encoder,
        inbox,
        websocket_rx,
        aliases,
    };

    Router::new()
        // FIXME: Remove API UIs which are not going to be used.
        .nest("/", Router::new().merge(Scalar::with_url("/scalar", ApiDoc::openapi())))
        .nest(
            "/",
            Router::new()
                .route("/startedz", get(checks::startedz))
                .route("/readyz", get(checks::readyz))
                .route("/healthyz", get(checks::healthyz))
                .with_state(state.into()),
        )
        .nest(
            BASE_PATH,
            axum::Router::new()
                .route("/aliases", get(alias::aliases))
                .route("/aliases", post(alias::set_alias))
                .route("/aliases/:alias", get(alias::get_alias))
                .route("/aliases/:alias", delete(alias::delete_alias))
                .route("/account/addresses", get(account::addresses))
                .route("/account/balances", get(account::balances))
                .route("/account/withdraw", get(account::withdraw))
                .route("/peers/:peerId", get(peers::show_peer_info))
                .route("/peers/:peerId/ping", post(peers::ping_peer))
                .route("/channels", get(channels::list_channels))
                .route("/channels", post(channels::open_channel))
                .route("/channels/:channelId", get(channels::show_channel))
                .route("/channels/:channelId", delete(channels::close_channel))
                .route("/channels/:channelId/fund", post(channels::fund_channel))
                .route("/channels/:channelId/tickets", get(tickets::show_channel_tickets))
                .route(
                    "/channels/:channelId/tickets/redeem",
                    post(tickets::redeem_tickets_in_channel),
                )
                .route(
                    "/channels/:channelId/tickets/aggregate",
                    post(tickets::aggregate_tickets_in_channel),
                )
                .route("/tickets", get(tickets::show_all_tickets))
                .route("/tickets/statistics", get(tickets::show_ticket_statistics))
                .route("/tickets/redeem", post(tickets::redeem_all_tickets))
                .route("/messages", post(messages::send_message))
                .route("/messages", delete(messages::delete_messages))
                .route("/messages/pop", post(messages::pop))
                .route("/messages/pop-all", post(messages::pop_all))
                .route("/messages/peek", post(messages::peek))
                .route("/messages/peek-all", post(messages::peek_all))
                .route("/messages/size", get(messages::size))
                .route("/messages/websocket", get(messages::websocket))
                .route("/network/price", get(network::price))
                .route("/node/version", get(node::version))
                .route("/node/configuration", get(node::configuration))
                .route("/node/info", get(node::info))
                .route("/node/peers", get(node::peers))
                .route("/node/entryNodes", get(node::entry_nodes))
                .route("/node/metrics", get(node::metrics))
                .with_state(inner_state.clone().into())
                .layer(middleware::from_fn_with_state(
                    inner_state,
                    token_authentication::authenticate,
                )),
        )
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(
                    CorsLayer::new()
                        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::DELETE])
                        .allow_origin(Any),
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
    InvalidPeerId,
    ChannelNotFound,
    TicketsNotFound,
    NotEnoughBalance,
    NotEnoughAllowance,
    ChannelAlreadyOpen,
    ChannelNotOpen,
    UnsupportedFeature,
    Timeout,
    Unauthorized,
    InvalidQuality,
    AliasAlreadyExists,
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
