//! REST API for the HOPRd node.
pub mod config;

use std::error::Error;
use std::str::FromStr;
use std::{collections::HashMap, sync::Arc};

use async_std::sync::RwLock;
use futures::StreamExt;
use futures_concurrency::stream::Merge;
use hopr_lib::TransportOutput;
use libp2p_identity::PeerId;
use log::{debug, error, warn};
use serde_json::json;
use serde_with::{serde_as, DisplayFromStr, DurationMilliSeconds};
use tide::http::headers::{HeaderName, AUTHORIZATION};
use tide::http::mime;
use tide::utils::async_trait;
use tide::{http::Mime, Request, Response};
use tide::{Middleware, Next, StatusCode};
use tide_websockets::{Message, WebSocket};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::Config;

use crate::config::Auth;
use hopr_lib::{
    errors::HoprLibError,
    {Address, Balance, BalanceType, Hopr},
};

pub const BASE_PATH: &str = "/api/v3";
pub const API_VERSION: &str = "3.0.0";

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::{MultiCounter, MultiHistogram};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_API_CALLS: MultiCounter = MultiCounter::new(
        "hopr_http_api_call_count",
        "Number of different REST API calls and their statuses",
        &["endpoint", "method", "status"]
    )
    .unwrap();
    static ref METRIC_COUNT_API_CALLS_TIMING: MultiHistogram = MultiHistogram::new(
        "hopr_http_api_call_timing_sec",
        "Timing of different REST API calls in seconds",
        vec![0.1, 0.25, 0.5, 1.0, 2.0, 5.0, 10.0],
        &["endpoint", "method"]
    )
    .unwrap();
}

#[derive(Clone)]
pub struct State<'a> {
    pub hopr: Arc<Hopr>,         // checks
    pub config: Arc<Config<'a>>, // swagger
}

pub type MessageEncoder = fn(&[u8]) -> Box<[u8]>;

#[derive(Clone)]
pub struct InternalState {
    pub auth: Arc<Auth>,
    pub hopr: Arc<Hopr>,
    pub inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    pub aliases: Arc<RwLock<HashMap<String, PeerId>>>,
    pub websocket_rx: async_broadcast::InactiveReceiver<TransportOutput>,
    pub msg_encoder: Option<MessageEncoder>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        checks::startedz,
        checks::readyz,
        checks::healthyz,
        alias::aliases,
        alias::set_alias,
        alias::get_alias,
        alias::delete_alias,
        account::addresses,
        account::balances,
        account::withdraw,
        channels::list_channels,
        channels::open_channel,
        channels::close_channel,
        channels::fund_channel,
        channels::show_channel,
        messages::send_message,
        messages::delete_messages,
        messages::size,
        messages::pop,
        messages::pop_all,
        messages::peek,
        messages::peek_all,
        network::price,
        tickets::show_channel_tickets,
        tickets::show_all_tickets,
        tickets::show_ticket_statistics,
        tickets::redeem_all_tickets,
        tickets::redeem_tickets_in_channel,
        tickets::aggregate_tickets_in_channel,
        node::version,
        node::peers,
        node::metrics,
        node::info,
        node::entry_nodes,
        peers::show_peer_info,
        peers::ping_peer
    ),
    components(
        schemas(
            ApiError,
            alias::PeerIdResponse, alias::AliasPeerIdBodyRequest,
            account::AccountAddressesResponse, account::AccountBalancesResponse, account::WithdrawBodyRequest,
            peers::NodePeerInfoResponse, peers::PingResponse,
            channels::ChannelsQueryRequest,channels::CloseChannelResponse, channels::OpenChannelBodyRequest, channels::OpenChannelResponse,
            channels::NodeChannel, channels::NodeChannelsResponse, channels::ChannelInfoResponse, channels::FundBodyRequest,
            messages::MessagePopResponse, messages::SendMessageResponse, messages::SendMessageBodyRequest, messages::SizeResponse, messages::TagQueryRequest, messages::GetMessageBodyRequest,
            messages::MessagePopAllResponse,
            tickets::NodeTicketStatisticsResponse, tickets::ChannelTicket,
            network::TicketPriceResponse,
            node::EntryNode, node::NodeInfoResponse, node::NodePeersQueryRequest,
            node::HeartbeatInfo, node::PeerInfo, node::NodePeersResponse, node::NodeVersionResponse
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Checks", description = "HOPR node functionality checks"),
        (name = "Alias", description = "HOPR node internal non-persistent alias endpoints"),
        (name = "Account", description = "HOPR node account endpoints"),
        (name = "Node", description = "HOPR node information endpoints"),
        (name = "Tickets", description = "HOPR node ticket management endpoints"),
        (name = "Messages", description = "HOPR node message manipulation endpoints"),
        (name = "Channels", description = "HOPR node chain channels manipulation endpoints"),
        (name = "Peers", description = "HOPR node peer manipulation endpoints"),
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
/// Token-based authentication middleware
struct TokenBasedAuthenticationMiddleware;

/// Implementation of the middleware
#[async_trait]
impl Middleware<InternalState> for TokenBasedAuthenticationMiddleware {
    async fn handle(&self, request: Request<InternalState>, next: Next<'_, InternalState>) -> tide::Result {
        let auth = request.state().auth.clone();

        let x_auth_header = HeaderName::from_str("x-auth-token").unwrap();

        let is_authorized = match auth.as_ref() {
            Auth::Token(expected_token) => {
                let auth_headers = request
                    .iter()
                    .filter_map(|(n, v)| (AUTHORIZATION.eq(n) || x_auth_header.eq(n)).then_some((n, v.as_str())))
                    .collect::<Vec<_>>();

                // Use "Authorization Bearer <token>" and "X-Auth-Token <token>" headers
                !auth_headers.is_empty()
                    && (auth_headers.contains(&(&AUTHORIZATION, &format!("Bearer {}", expected_token)))
                        || auth_headers.contains(&(&x_auth_header, &expected_token)))
            }
            Auth::None => true,
        };

        if !is_authorized {
            let reject_response = Response::builder(StatusCode::Unauthorized)
                .content_type(mime::JSON)
                .body(ApiErrorStatus::Unauthorized)
                .build();

            return Ok(reject_response);
        }

        // Go forward to the next middleware or request handler
        Ok(next.run(request).await)
    }
}

/// Custom request logging middleware
struct LogRequestMiddleware(log::Level);

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Middleware<T> for LogRequestMiddleware {
    async fn handle(&self, mut req: Request<T>, next: Next<'_, T>) -> tide::Result {
        struct LogMiddlewareHasBeenRun;

        if req.ext::<LogMiddlewareHasBeenRun>().is_some() {
            return Ok(next.run(req).await);
        }
        req.set_ext(LogMiddlewareHasBeenRun);

        let peer_addr = req.peer_addr().map(String::from);
        let path = req.url().path().to_owned();
        let method = req.method().to_string();

        let start = std::time::Instant::now();
        let response = next.run(req).await;
        let response_duration = start.elapsed();

        let status = response.status();

        #[cfg(all(feature = "prometheus", not(test)))]
        {
            // We're not interested on metrics for other endpoints
            if path.starts_with("/api/v3/") {
                METRIC_COUNT_API_CALLS.increment(&[&path, &method, &status.to_string()]);
                METRIC_COUNT_API_CALLS_TIMING.observe(&[&path, &method], response_duration.as_secs_f64());
            }
        }

        log::log!(
            self.0,
            r#"{} "{method} {path}" {status} {} {}ms"#,
            peer_addr.as_deref().unwrap_or("-"),
            response
                .len()
                .map(|l| l.to_string())
                .unwrap_or_else(|| String::from("-")),
            response_duration.as_millis()
        );
        Ok(response)
    }
}

async fn serve_swagger(request: tide::Request<State<'_>>) -> tide::Result<Response> {
    let config = request.state().config.clone();
    let path = request.url().path().to_string();
    let tail = path.strip_prefix("/swagger-ui/").unwrap();

    match utoipa_swagger_ui::serve(tail, config) {
        Ok(swagger_file) => swagger_file
            .map(|file| {
                Ok(Response::builder(200)
                    .body(file.bytes.to_vec())
                    .content_type(file.content_type.parse::<Mime>()?)
                    .build())
            })
            .unwrap_or_else(|| Ok(Response::builder(404).build())),
        Err(error) => Ok(Response::builder(500).body(error.to_string()).build()),
    }
}

enum WebSocketInput {
    Network(TransportOutput),
    WsInput(std::result::Result<tide_websockets::Message, tide_websockets::Error>),
}

pub async fn run_hopr_api(
    host: &str,
    cfg: &crate::config::Api,
    hopr: Arc<hopr_lib::Hopr>,
    inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    websocket_rx: async_broadcast::InactiveReceiver<TransportOutput>,
    msg_encoder: Option<MessageEncoder>,
) {
    // Prepare alias part of the state
    let aliases: Arc<RwLock<HashMap<String, PeerId>>> = Arc::new(RwLock::new(HashMap::new()));
    aliases.write().await.insert("me".to_owned(), hopr.me_peer_id());

    let state = State {
        hopr,
        config: Arc::new(Config::from("/api-docs/openapi.json")),
    };

    let mut app = tide::with_state(state.clone());

    app.with(LogRequestMiddleware(log::Level::Debug));

    app.at("/api-docs/openapi.json")
        .get(|_| async move { Ok(Response::builder(200).body(json!(ApiDoc::openapi()))) });

    app.at("/swagger-ui/*").get(serve_swagger);

    app.at("/startedz").get(checks::startedz);
    app.at("/readyz").get(checks::readyz);
    app.at("/healthyz").get(checks::healthyz);

    app.at(BASE_PATH).nest({
        let mut api = tide::with_state(InternalState {
            auth: Arc::new(cfg.auth.clone()),
            hopr: state.hopr.clone(),
            msg_encoder,
            inbox,
            websocket_rx,
            aliases,
        });

        api.with(TokenBasedAuthenticationMiddleware);

        api.at("/aliases").get(alias::aliases).post(alias::set_alias);

        api.at("/aliases/:alias")
            .get(alias::get_alias)
            .delete(alias::delete_alias);

        api.at("/account/addresses").get(account::addresses);
        api.at("/account/balances").get(account::balances);
        api.at("/account/withdraw").get(account::withdraw);

        api.at("/peers/:peerId")
            .get(peers::show_peer_info)
            .at("/ping")
            .post(peers::ping_peer);

        api.at("/channels")
            .get(channels::list_channels)
            .post(channels::open_channel);

        api.at("/channels/:channelId")
            .get(channels::show_channel)
            .delete(channels::close_channel);

        api.at("/channels/:channelId/fund").post(channels::fund_channel);

        api.at("/channels/:channelId/tickets")
            .get(tickets::show_channel_tickets);

        api.at("/channels/:channelId/tickets/redeem")
            .post(tickets::redeem_tickets_in_channel);

        api.at("/channels/:channelId/tickets/aggregate")
            .post(tickets::aggregate_tickets_in_channel);

        api.at("/tickets").get(tickets::show_all_tickets);
        api.at("/tickets/statistics").get(tickets::show_ticket_statistics);
        api.at("/tickets/redeem").post(tickets::redeem_all_tickets);

        api.at("/messages")
            .post(messages::send_message)
            .delete(messages::delete_messages);
        api.at("/messages/pop").post(messages::pop);
        api.at("/messages/pop-all").post(messages::pop_all);
        api.at("/messages/peek").post(messages::peek);
        api.at("/messages/peek-all").post(messages::peek_all);
        api.at("/messages/size").get(messages::size);
        api.at("/messages/websocket")
            .get(WebSocket::new(|request: Request<InternalState>, ws_con| {
                let ws_rx = request.state().websocket_rx.activate_cloned();

                async move {
                    let mut queue = (
                        ws_con.clone().map(WebSocketInput::WsInput),
                        ws_rx.map(WebSocketInput::Network),
                    )
                        .merge();

                    while let Some(v) = queue.next().await {
                        match v {
                            WebSocketInput::Network(net_in) => match net_in {
                                TransportOutput::Received(data) => {
                                    debug!("websocket notifying client with received msg {data}");
                                    ws_con.send_json(&json!(messages::WebSocketReadMsg::from(data))).await?;
                                }
                                TransportOutput::Sent(hkc) => {
                                    debug!("websocket notifying client with received ack {hkc}");
                                    ws_con
                                        .send_json(&json!(messages::WebSocketReadAck::from_ack(hkc)))
                                        .await?;
                                }
                            },
                            WebSocketInput::WsInput(ws_in) => match ws_in {
                                Ok(Message::Text(input)) => {
                                    let data: messages::WebSocketSendMsg = serde_json::from_str(&input)?;
                                    if data.cmd == "sendmsg" {
                                        let hopr = request.state().hopr.clone();

                                        // Use the message encoder, if any
                                        // TODO: remove RLP in 3.0
                                        let msg_body = request
                                            .state()
                                            .msg_encoder
                                            .as_ref()
                                            .map(|enc| enc(data.args.body.as_bytes()))
                                            .unwrap_or_else(|| Box::from(data.args.body.as_bytes()));

                                        let hkc = hopr
                                            .send_message(
                                                // data.args.body.into_bytes().into_boxed_slice(),
                                                msg_body,
                                                data.args.peer_id,
                                                data.args.path,
                                                data.args.hops,
                                                Some(data.args.tag),
                                            )
                                            .await?;

                                        debug!("websocket notifying client with sent ack {hkc}");
                                        ws_con
                                            .send_json(&json!(messages::WebSocketReadAck::from_ack_challenge(hkc)))
                                            .await?;
                                    } else {
                                        warn!("skipping an unsupported websocket command '{}'", data.cmd);
                                    }
                                }
                                Ok(_) => {
                                    warn!("encountered an unsupported websocket input type");
                                }
                                Err(e) => error!("failed to get a valid websocket message: {e}"),
                            },
                        }
                    }

                    Ok(())
                }
            }));

        api.at("/network/price").get(network::price);

        api.at("/node/version").get(node::version);
        api.at("/node/info").get(node::info);
        api.at("/node/peers").get(node::peers);
        api.at("/node/entryNodes").get(node::entry_nodes);
        api.at("/node/metrics").get(node::metrics);

        api
    });

    app.listen(host)
        .await
        .expect("the REST API server should run successfully")
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
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

impl From<ApiErrorStatus> for tide::Body {
    fn from(value: ApiErrorStatus) -> Self {
        json!(ApiError::from(value)).into()
    }
}

// Errors lead to `UnknownFailure` per default
impl<T: Error> From<T> for ApiErrorStatus {
    fn from(value: T) -> Self {
        Self::UnknownFailure(value.to_string())
    }
}

mod alias {
    use super::*;

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "peerId": "12D3KooWRWeTozREYHzWTbuCYskdYhED1MXpDwTrmccwzFrd2mEA"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct PeerIdResponse {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub peer_id: PeerId,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "alias": "Alice",
        "peerId": "12D3KooWRWeTozREYHzWTbuCYskdYhED1MXpDwTrmccwzFrd2mEA"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct AliasPeerIdBodyRequest {
        pub alias: String,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub peer_id: PeerId,
    }

    /// Get each previously set alias and its corresponding PeerId
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/aliases"),
        responses(
            (status = 200, description = "Each alias with its corresponding PeerId", body = HashMap<String, String>, example = json!({
                    "alice": "12D3KooWPWD5P5ZzMRDckgfVaicY5JNoo7JywGotoAv17d7iKx1z",
                    "me": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS"
            })),
            (status = 401, description = "Invalid authorization token.", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Alias",
    )]
    pub async fn aliases(req: Request<InternalState>) -> tide::Result<Response> {
        let aliases = req.state().aliases.clone();

        let aliases = aliases
            .read()
            .await
            .iter()
            .map(|(key, value)| (key.clone(), value.to_string()))
            .collect::<HashMap<String, String>>();

        Ok(Response::builder(200).body(json!(aliases)).build())
    }

    /// Set alias for a peer with a specific PeerId.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/aliases"),
        request_body(
            content = AliasPeerIdBodyRequest,
            description = "Alias name along with the PeerId to be aliased",
            content_type = "application/json"),
        responses(
            (status = 201, description = "Alias set successfully.", body = PeerIdResponse),
            (status = 400, description = "Invalid PeerId: The format or length of the peerId is incorrect.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Alias",
    )]
    pub async fn set_alias(mut req: Request<InternalState>) -> tide::Result<Response> {
        let args: AliasPeerIdBodyRequest = req.body_json().await?;
        let aliases = req.state().aliases.clone();

        aliases.write().await.insert(args.alias, args.peer_id);
        Ok(Response::builder(200)
            .body(json!(PeerIdResponse { peer_id: args.peer_id }))
            .build())
    }

    /// Get alias for the PeerId (Hopr address) that have this alias assigned to it.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/aliases/{{alias}}"),
        params(
            ("alias" = String, Path, description = "Alias to be shown"),
        ),
        responses(
            (status = 200, description = "Get PeerId for an alias", body = PeerIdResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "PeerId not found", body = ApiError),
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Alias",
    )]
    pub async fn get_alias(req: Request<InternalState>) -> tide::Result<Response> {
        let alias = req.param("alias")?.parse::<String>()?;
        let aliases = req.state().aliases.clone();

        let aliases = aliases.read().await;
        if let Some(peer_id) = aliases.get(&alias) {
            Ok(Response::builder(200)
                .body(json!(PeerIdResponse { peer_id: *peer_id }))
                .build())
        } else {
            Ok(Response::builder(404).body(ApiErrorStatus::InvalidInput).build())
        }
    }

    /// Delete an alias.
    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/aliases/{{alias}}"),
        params(
            ("alias" = String, Path, description = "Alias to be shown"),
        ),
        responses(
            (status = 204, description = "Alias removed successfully"),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)   // This can never happen
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Alias",
    )]
    pub async fn delete_alias(req: Request<InternalState>) -> tide::Result<Response> {
        let alias = req.param("alias")?.parse::<String>()?;
        let aliases = req.state().aliases.clone();

        let _ = aliases.write().await.remove(&alias);

        Ok(Response::builder(204).build())
    }
}

mod account {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "hopr": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS",
        "native": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct AccountAddressesResponse {
        pub native: String,
        pub hopr: String,
    }

    /// Get node's HOPR and native addresses.
    ///
    /// HOPR address is represented by the P2P PeerId and can be used by other node owner to interact with this node.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/account/addresses"),
        responses(
            (status = 200, description = "The node's public addresses", body = AccountAddressesResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Account",
    )]
    pub(super) async fn addresses(req: Request<InternalState>) -> tide::Result<Response> {
        let addresses = AccountAddressesResponse {
            native: req.state().hopr.me_onchain().to_string(),
            hopr: req.state().hopr.me_peer_id().to_string(),
        };

        Ok(Response::builder(200).body(json!(addresses)).build())
    }

    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "hopr": "2000000000000000000000 HOPR",
        "native": "9999563581204904000 Native",
        "safeHopr": "2000000000000000000000 HOPR",
        "safeHoprAllowance": "115792089237316195423570985008687907853269984665640564039457584007913129639935 HOPR",
        "safeNative": "10000000000000000000 Native"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct AccountBalancesResponse {
        pub safe_native: String,
        pub native: String,
        pub safe_hopr: String,
        pub hopr: String,
        pub safe_hopr_allowance: String,
    }

    /// Get node's and associated Safe's HOPR and native balances as the allowance for HOPR
    /// tokens to be drawn by HoprChannels from Safe.
    ///
    /// HOPR tokens from the Safe balance are used to fund the payment channels between this
    /// node and other nodes on the network.
    /// NATIVE balance of the Node is used to pay for the gas fees for the blockchain.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/account/balances"),
        responses(
            (status = 200, description = "The node's HOPR and Safe balances", body = AccountBalancesResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Account",
    )]
    pub(super) async fn balances(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let mut account_balances = AccountBalancesResponse::default();

        match hopr.get_balance(BalanceType::Native).await {
            Ok(v) => account_balances.native = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }

        match hopr.get_balance(BalanceType::HOPR).await {
            Ok(v) => account_balances.hopr = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }

        match hopr.get_safe_balance(BalanceType::Native).await {
            Ok(v) => account_balances.safe_native = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }

        match hopr.get_safe_balance(BalanceType::HOPR).await {
            Ok(v) => account_balances.safe_hopr = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }

        match hopr.safe_allowance().await {
            Ok(v) => account_balances.safe_hopr_allowance = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }

        Ok(Response::builder(200).body(json!(account_balances)).build())
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "address": "0xb4ce7e6e36ac8b01a974725d5ba730af2b156fbe",
        "amount": 20000,
        "currency": "HOPR"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct WithdrawBodyRequest {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        currency: BalanceType,
        amount: u128,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        address: Address,
    }

    /// Withdraw funds from this node to the ethereum wallet address.
    ///
    /// Both NATIVE or HOPR can be withdrawn using this method.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/account/withdraw"),
        request_body(
            content = WithdrawBodyRequest,
            content_type = "application/json"),
        responses(
            (status = 200, description = "The node's funds have been withdrawn", body = AccountBalancesResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Account",
    )]
    pub(super) async fn withdraw(mut req: Request<InternalState>) -> tide::Result<Response> {
        let withdraw_req_data: WithdrawBodyRequest = req.body_json().await?;

        match req
            .state()
            .hopr
            .withdraw(
                withdraw_req_data.address,
                Balance::new(withdraw_req_data.amount, withdraw_req_data.currency),
            )
            .await
        {
            Ok(receipt) => Ok(Response::builder(200).body(json!({"receipt": receipt})).build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }
}

mod peers {
    use super::*;
    use hopr_lib::{HoprTransportError, Multiaddr, PEER_METADATA_PROTOCOL_VERSION};
    use serde_with::DurationMilliSeconds;
    use std::str::FromStr;
    use std::time::Duration;

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "announced": [
        "/ip4/10.0.2.100/tcp/19093"
        ],
        "observed": [
        "/ip4/10.0.2.100/tcp/19093"
        ]
    }))]
    pub(crate) struct NodePeerInfoResponse {
        #[serde_as(as = "Vec<DisplayFromStr>")]
        #[schema(value_type = Vec<String>)]
        pub announced: Vec<Multiaddr>,
        #[serde_as(as = "Vec<DisplayFromStr>")]
        #[schema(value_type = Vec<String>)]
        pub observed: Vec<Multiaddr>,
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/peers/{{peerId}}"),
        params(
            ("peerId" = String, Path, description = "PeerID of the requested peer")
        ),
        responses(
            (status = 200, description = "Peer information fetched successfully.", body = NodePeerInfoResponse),
            (status = 400, description = "Invalid peer id", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Peers",
    )]
    pub(super) async fn show_peer_info(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match PeerId::from_str(req.param("peerId")?) {
            Ok(peer) => Ok(Response::builder(200)
                .body(json!(NodePeerInfoResponse {
                    announced: hopr.multiaddresses_announced_to_dht(&peer).await,
                    observed: hopr.network_observed_multiaddresses(&peer).await
                }))
                .build()),
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidPeerId).build()),
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "latency": 200,
        "reportedVersion": "2.1.0"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct PingResponse {
        #[serde_as(as = "DurationMilliSeconds<u64>")]
        #[schema(value_type = u64)]
        pub latency: std::time::Duration,
        pub reported_version: String,
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/peers/{{peerId}}/ping"),
        params(
            ("peerId" = String, Path, description = "PeerID of the requested peer")
        ),
        responses(
            (status = 200, description = "Ping successful", body = PingResponse),
            (status = 400, description = "Invalid peer id", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Peers",
    )]
    pub(super) async fn ping_peer(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match PeerId::from_str(req.param("peerId")?) {
            Ok(peer) => match hopr.ping(&peer).await {
                Ok(latency) => Ok(Response::builder(200)
                    .body(json!(PingResponse {
                        latency: latency.unwrap_or(Duration::ZERO), // TODO: what should be the correct default ?
                        reported_version: hopr
                            .network_peer_info(&peer)
                            .await
                            .and_then(|s| s.metadata().get(PEER_METADATA_PROTOCOL_VERSION).cloned())
                            .unwrap_or("unknown".into())
                    }))
                    .build()),
                Err(HoprLibError::TransportError(HoprTransportError::Protocol(hopr_lib::ProtocolError::Timeout))) => {
                    Ok(Response::builder(422).body(ApiErrorStatus::Timeout).build())
                }
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidPeerId).build()),
        }
    }
}

mod channels {
    use super::*;
    use futures::TryFutureExt;
    use hopr_crypto_types::types::Hash;
    use hopr_lib::{ChannelEntry, ChannelStatus, CoreEthereumActionsError, ToHex};

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct NodeChannel {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub id: Hash,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub peer_address: Address,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub status: ChannelStatus,
        pub balance: String,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "balance": "10000000000000000000",
        "channelEpoch": 1,
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "closureTime": 0,
        "destinationAddress": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
        "destinationPeerId": "12D3KooWPWD5P5ZzMRDckgfVaicY5JNoo7JywGotoAv17d7iKx1z",
        "sourceAddress": "0x07eaf07d6624f741e04f4092a755a9027aaab7f6",
        "sourcePeerId": "12D3KooWJmLm8FnBfvYQ5BAZ5qcYBxQFFBzAAEYUBUNJNE8cRsYS",
        "status": "Open",
        "ticketIndex": 0
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct ChannelInfoResponse {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub channel_id: Hash,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub source_address: Address,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub destination_address: Address,
        pub source_peer_id: String,
        pub destination_peer_id: String,
        pub balance: String,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub status: ChannelStatus,
        pub ticket_index: u32,
        pub channel_epoch: u32,
        pub closure_time: u64,
    }

    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "all": [],
        "incoming": [],
        "outgoing": [
        {
            "balance": "10000000000000000010",
            "id": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
            "peerAddress": "0x188c4462b75e46f0c7262d7f48d182447b93a93c",
            "status": "Open"
        }
        ]
    }))]
    pub(crate) struct NodeChannelsResponse {
        pub incoming: Vec<NodeChannel>,
        pub outgoing: Vec<NodeChannel>,
        pub all: Vec<ChannelInfoResponse>,
    }

    async fn query_topology_info(channel: &ChannelEntry, node: &Hopr) -> Result<ChannelInfoResponse, HoprLibError> {
        Ok(ChannelInfoResponse {
            channel_id: channel.get_id(),
            source_address: channel.source,
            destination_address: channel.destination,
            source_peer_id: node
                .chain_key_to_peerid(&channel.source)
                .await?
                .map(PeerId::to_string)
                .unwrap_or_else(|_| {
                    warn!("failed to map {} to peerid", channel.source);
                    "".into()
                }),
            destination_peer_id: node
                .chain_key_to_peerid(&channel.destination)
                .await?
                .map(PeerId::to_string)
                .unwrap_or_else(|_| {
                    warn!("failed to map {} to peerid", channel.destination);
                    "".into()
                }),
            balance: channel.balance.amount().to_string(),
            status: channel.status,
            ticket_index: channel.ticket_index.as_u32(),
            channel_epoch: channel.channel_epoch.as_u32(),
            closure_time: channel.closure_time.as_u64(),
        })
    }

    #[derive(Debug, Default, Copy, Clone, serde::Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
    #[into_params(parameter_in = Query)]
    #[serde(default, rename_all = "camelCase")]
    pub(crate) struct ChannelsQueryRequest {
        #[schema(required = false)]
        #[serde(default)]
        pub including_closed: bool,
        #[schema(required = false)]
        #[serde(default)]
        pub full_topology: bool,
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        params(ChannelsQueryRequest),
        responses(
            (status = 200, description = "Channels fetched successfully", body = NodeChannelsResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels",
    )]
    pub(super) async fn list_channels(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        let query: ChannelsQueryRequest = req.query()?;

        if query.full_topology {
            let hopr_clone = hopr.clone();
            let topology = hopr
                .all_channels()
                .and_then(|channels| async move {
                    futures::future::try_join_all(channels.iter().map(|c| query_topology_info(c, hopr_clone.as_ref())))
                        .await
                })
                .await;

            match topology {
                Ok(all) => Ok(Response::builder(200)
                    .body(json!(NodeChannelsResponse {
                        incoming: vec![],
                        outgoing: vec![],
                        all
                    }))
                    .build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            }
        } else {
            let channels = hopr
                .channels_to(&hopr.me_onchain())
                .and_then(|incoming| async {
                    let outgoing = hopr.channels_from(&hopr.me_onchain()).await?;
                    Ok((incoming, outgoing))
                })
                .await;

            match channels {
                Ok((incoming, outgoing)) => {
                    let channel_info = NodeChannelsResponse {
                        incoming: incoming
                            .into_iter()
                            .filter(|c| query.including_closed || c.status != ChannelStatus::Closed)
                            .map(|c| NodeChannel {
                                    id: c.get_id(),
                                    peer_address: c.source,
                                    status: c.status,
                                    balance: c.balance.amount().to_string(),
                            })
                            .collect(),
                        outgoing: outgoing
                            .into_iter()
                            .filter(|c| query.including_closed || c.status != ChannelStatus::Closed)
                            .map(|c| NodeChannel {
                                id: c.get_id(),
                                peer_address: c.destination,
                                status: c.status,
                                balance: c.balance.amount().to_string(),
                            })
                            .collect(),
                        all: vec![],
                    };

                    Ok(Response::builder(200).body(json!(channel_info)).build())
                }
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            }
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[schema(example = json!({
        "amount": "10",
        "peerAddress": "0xa8194d36e322592d4c707b70dbe96121f5c74c64"
    }))]
    pub(crate) struct OpenChannelBodyRequest {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub peer_address: Address,
        pub amount: String,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "transactionReceipt": "0x5181ac24759b8e01b3c932e4636c3852f386d17517a8dfc640a5ba6f2258f29c"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct OpenChannelResponse {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub channel_id: Hash,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub transaction_receipt: Hash,
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        request_body(
            content = OpenChannelBodyRequest,
            description = "Open channel request specification",
            content_type = "application/json"),
        responses(
            (status = 201, description = "Channel successfully opened", body = OpenChannelResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 403, description = "Failed to open the channel because of insufficient HOPR balance or allowance.", body = ApiError),
            (status = 409, description = "Failed to open the channel because the channel between this nodes already exists.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels",
    )]
    pub(super) async fn open_channel(mut req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let open_req: OpenChannelBodyRequest = req.body_json().await?;

        match hopr
            .open_channel(
                &open_req.peer_address,
                &Balance::new_from_str(&open_req.amount, BalanceType::HOPR),
            )
            .await
        {
            Ok(channel_details) => Ok(Response::builder(201)
                .body(json!(OpenChannelResponse {
                    channel_id: channel_details.channel_id,
                    transaction_receipt: channel_details.tx_hash
                }))
                .build()),
            Err(HoprLibError::ChainError(CoreEthereumActionsError::BalanceTooLow)) => {
                Ok(Response::builder(403).body(ApiErrorStatus::NotEnoughBalance).build())
            }
            Err(HoprLibError::ChainError(CoreEthereumActionsError::NotEnoughAllowance)) => {
                Ok(Response::builder(403).body(ApiErrorStatus::NotEnoughAllowance).build())
            }
            Err(HoprLibError::ChainError(CoreEthereumActionsError::ChannelAlreadyExists)) => {
                Ok(Response::builder(409).body(ApiErrorStatus::ChannelAlreadyOpen).build())
            }
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 200, description = "Channel fetched successfully", body = ChannelInfoResponse),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels",
    )]
    pub(super) async fn show_channel(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.channel_from_hash(&channel_id).await {
                Ok(Some(channel)) => Ok(Response::builder(200)
                    .body(json!(query_topology_info(&channel, hopr.as_ref()).await?))
                    .build()),
                Ok(None) => Ok(Response::builder(404).body(ApiErrorStatus::ChannelNotFound).build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidChannelId).build()),
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "channelStatus": "PendingToClose",
        "receipt": "0xd77da7c1821249e663dead1464d185c03223d9663a06bc1d46ed0ad449a07118"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CloseChannelResponse {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub receipt: Hash,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub channel_status: ChannelStatus,
    }

    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 200, description = "Channel closed successfully", body = CloseChannelResponse),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels",
    )]
    pub(super) async fn close_channel(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.close_channel_by_id(channel_id, false).await {
                Ok(receipt) => Ok(Response::builder(200)
                    .body(json!(CloseChannelResponse {
                        channel_status: receipt.status,
                        receipt: receipt.tx_hash
                    }))
                    .build()),
                Err(HoprLibError::ChainError(CoreEthereumActionsError::ChannelDoesNotExist)) => {
                    Ok(Response::builder(404).body(ApiErrorStatus::ChannelNotFound).build())
                }
                Err(HoprLibError::ChainError(CoreEthereumActionsError::InvalidArguments(_))) => {
                    Ok(Response::builder(422).body(ApiErrorStatus::UnsupportedFeature).build())
                }
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidChannelId).build()),
        }
    }

    #[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "amount": "1000"
    }))]
    pub(crate) struct FundBodyRequest {
        pub amount: String,
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/fund"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        request_body(
            content = FundBodyRequest,
            description = "Amount of HOPR to fund the channel",
            content_type = "application/json",
        ),
        responses(
            (status = 200, description = "Channel funded successfully", body = String),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels",
    )]
    pub(super) async fn fund_channel(mut req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let fund_req: FundBodyRequest = req.body_json().await?;
        let amount = Balance::new_from_str(&fund_req.amount, BalanceType::HOPR);

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.fund_channel(&channel_id, &amount).await {
                Ok(hash) => Ok(Response::builder(200).body(hash.to_string()).build()),
                Err(HoprLibError::ChainError(CoreEthereumActionsError::ChannelDoesNotExist)) => {
                    Ok(Response::builder(404).body(ApiErrorStatus::ChannelNotFound).build())
                }
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidChannelId).build()),
        }
    }
}

mod messages {
    use std::time::Duration;

    use hopr_lib::HalfKeyChallenge;

    use super::*;

    #[derive(Debug, Default, Clone, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
    #[into_params(parameter_in = Query)]
    pub(crate) struct TagQueryRequest {
        #[schema(required = false)]
        #[serde(default)]
        pub tag: Option<u16>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    pub(crate) struct SizeResponse {
        pub size: usize,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Deserialize, validator::Validate, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    #[schema(example = json!({
        "body": "Test message",
        "hops": 1,
        "path": [
            "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33"
        ],
        "peerId": "12D3KooWEDc1vGJevww48trVDDf6pr1f6N3F86sGJfQrKCyc8kJ1",
        "tag": 20
    }))]
    pub(crate) struct SendMessageBodyRequest {
        /// The message tag used to filter messages based on application
        pub tag: u16,
        /// Message to be transmitted over the network
        pub body: String,
        /// The recipient HOPR PeerId
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub peer_id: PeerId,
        #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
        // #[validate(length(min=0, max=3))]        // NOTE: issue in serde_as with validator -> no order is correct
        #[schema(value_type = Option<Vec<String>>)]
        pub path: Option<Vec<PeerId>>,
        #[validate(range(min = 1, max = 3))]
        pub hops: Option<u16>,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "challenge": "031916ee5bfc0493f40c353a670fc586a3a28f9fce9cd065ff9d1cbef19b46eeba"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct SendMessageResponse {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub challenge: HalfKeyChallenge,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        #[schema(value_type = u64)]
        pub timestamp: std::time::Duration,
    }

    #[serde_as]
    #[derive(Debug, Default, Clone, serde::Deserialize, utoipa::ToSchema)]
    pub(crate) struct GetMessageBodyRequest {
        /// The message tag used to filter messages based on application
        #[schema(required = false)]
        #[serde(default)]
        pub tag: Option<u16>,
        /// Timestamp to filter messages received after this timestamp
        #[serde_as(as = "Option<DurationMilliSeconds<u64>>")]
        #[schema(required = false, value_type = u64)]
        #[serde(default)]
        pub timestamp: Option<std::time::Duration>,
    }

    /// Send a message to another peer using a given path.
    ///
    /// The message can be sent either over a specified path or using a specified
    /// number of HOPS, if no path is given.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/messages"),
        request_body(
            content = SendMessageBodyRequest,
            description = "Body of a message to send",
            content_type = "application/json"),
        responses(
            (status = 202, description = "The message was sent successfully, DOES NOT imply successful delivery.", body = SendMessageResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Messages",
    )]
    pub async fn send_message(mut req: Request<InternalState>) -> tide::Result<Response> {
        let args: SendMessageBodyRequest = req.body_json().await?;
        let hopr = req.state().hopr.clone();

        // Use the message encoder, if any
        let msg_body = req
            .state()
            .msg_encoder
            .as_ref()
            .map(|enc| enc(args.body.as_bytes()))
            .unwrap_or_else(|| Box::from(args.body.as_bytes()));

        if let Some(path) = &args.path {
            if path.len() > 3 {
                return Ok(Response::builder(422)
                    .body(ApiErrorStatus::UnknownFailure(
                        "The path components must contain at most 3 elements".into(),
                    ))
                    .build());
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();

        match hopr
            .send_message(msg_body, args.peer_id, args.path, args.hops, Some(args.tag))
            .await
        {
            Ok(challenge) => Ok(Response::builder(202)
                .body(json!(SendMessageResponse { challenge, timestamp }))
                .build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }

    #[derive(Debug, Default, Clone, serde::Deserialize, utoipa::ToSchema)]
    #[schema(value_type = String)] //, format = Binary)]
    pub struct Text(String);

    #[derive(Debug, Clone, serde::Deserialize)]
    pub(crate) struct WebSocketSendMsg {
        pub cmd: String,
        pub args: SendMessageBodyRequest,
    }

    #[derive(Debug, Clone, serde::Serialize)]
    pub(crate) struct WebSocketReadMsg {
        #[serde(rename = "type")]
        type_: String,
        pub tag: u16,
        pub body: String,
    }

    impl From<hopr_lib::ApplicationData> for WebSocketReadMsg {
        fn from(value: hopr_lib::ApplicationData) -> Self {
            Self {
                type_: "message".into(),
                tag: value.application_tag.unwrap_or(0),
                // TODO: Byte order structures should be used instead of the String object
                body: String::from_utf8_lossy(value.plain_text.as_ref()).to_string(),
            }
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize)]
    pub(crate) struct WebSocketReadAck {
        #[serde(rename = "type")]
        type_: String,
        #[serde_as(as = "DisplayFromStr")]
        pub id: HalfKeyChallenge,
    }

    impl WebSocketReadAck {
        pub fn from_ack(value: HalfKeyChallenge) -> Self {
            Self {
                type_: "message-ack".into(),
                id: value,
            }
        }

        pub fn from_ack_challenge(value: HalfKeyChallenge) -> Self {
            Self {
                type_: "message-ack-challenge".into(),
                id: value,
            }
        }
    }

    /// Websocket endpoint exposing a subset of message functions.
    ///
    /// Incoming messages from other nodes are sent to the websocket client.
    ///
    /// The following message can be set to the server by the client:
    /// ```json
    /// {
    ///     cmd: "sendmsg",
    ///     args: {
    ///         peerId: "SOME_PEER_ID",
    ///         path: [],
    ///         hops: 1,
    ///         body: "asdasd",
    ///         tag: 2
    ///     }
    /// }
    /// ```
    ///
    /// The command arguments follow the same semantics as in the dedicated API endpoint for sending messages.
    ///
    /// The following messages may be sent by the server over the Websocket connection:
    /// ````json
    /// {
    ///   type: "message",
    ///   tag: 12,
    ///   body: "my example message"
    /// }
    ///
    /// {
    ///   type: "message-ack",
    ///   id: "some challenge id"
    /// }
    ///
    /// {
    ///   type: "message-ack-challenge",
    ///   id: "some challenge id"
    /// }
    ///
    /// Authentication (if enabled) is done by cookie `X-Auth-Token`.
    ///
    /// Connect to the endpoint by using a WS client. No preview available. Example: `ws://127.0.0.1:3001/api/v3/messages/websocket
    #[allow(dead_code)] // not dead code, just for documentation
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/messages/websocket"),
        responses(
            (status = 206, description = "Incoming data", body = Text, content_type = "application/text"),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Messages",
    )]
    pub async fn websocket(_req: Request<InternalState>) -> tide::Result<Response> {
        // Dummy implementation for utoipa, the websocket is created in-place inside the tide server
        Ok(Response::builder(422)
            .body(ApiErrorStatus::UnknownFailure("unimplemented".into()))
            .build())
    }

    /// Delete messages from nodes message inbox.
    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/messages"),
        params(TagQueryRequest),
        responses(
            (status = 204, description = "Messages successfully deleted."),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        tag = "Messages",
        security(
            ("api_token" = [])
        )
    )]
    pub async fn delete_messages(req: Request<InternalState>) -> tide::Result<Response> {
        let tag: TagQueryRequest = req.query()?;
        let inbox = req.state().inbox.clone();

        inbox.write().await.pop_all(tag.tag).await;
        Ok(Response::builder(204).build())
    }

    /// Get size of filtered message inbox for a specific tag
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/messages/size"),
        params(TagQueryRequest),
        responses(
            (status = 200, description = "Returns the message inbox size filtered by the given tag", body = SizeResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Messages"
    )]
    pub async fn size(req: Request<InternalState>) -> tide::Result<Response> {
        let query: TagQueryRequest = req.query()?;
        let inbox = req.state().inbox.clone();

        let size = inbox.read().await.size(query.tag).await;

        Ok(Response::builder(200).body(json!(SizeResponse { size })).build())
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "body": "Test message 1",
        "receivedAt": 1704453953073i64,
        "tag": 20
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct MessagePopResponse {
        tag: u16,
        body: String,
        #[serde_as(as = "DurationMilliSeconds<u64>")]
        #[schema(value_type = u64)]
        received_at: std::time::Duration,
    }

    fn to_api_message(data: hopr_lib::ApplicationData, received_at: Duration) -> Result<MessagePopResponse, String> {
        if let Some(tag) = data.application_tag {
            match std::str::from_utf8(&data.plain_text) {
                Ok(data_str) => Ok(MessagePopResponse {
                    tag,
                    body: data_str.into(),
                    received_at,
                }),
                Err(error) => Err(format!("Failed to deserialize data into string: {error}")),
            }
        } else {
            Err("No application tag was present despite picking from a tagged inbox".into())
        }
    }

    /// Get the oldest message currently present in the nodes message inbox.
    ///
    /// The message is removed from the inbox.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/messages/pop"),
        request_body(
            content = TagQueryRequest,
            description = "Tag of message queue to pop from",
            content_type = "application/json"
        ),
        responses(
            (status = 200, description = "Message successfully extracted.", body = MessagePopResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Messages"
    )]
    pub async fn pop(mut req: Request<InternalState>) -> tide::Result<Response> {
        let tag: TagQueryRequest = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        if let Some((data, ts)) = inbox.pop(tag.tag).await {
            match to_api_message(data, ts) {
                Ok(message) => Ok(Response::builder(200).body(json!(message)).build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::UnknownFailure(e)).build()),
            }
        } else {
            Ok(Response::builder(404).build())
        }
    }

    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    pub(crate) struct MessagePopAllResponse {
        pub messages: Vec<MessagePopResponse>,
    }

    /// Get the list of messages currently present in the nodes message inbox.
    ///
    /// The messages are removed from the inbox.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/messages/pop-all"),
        request_body(
            content = TagQueryRequest,
            description = "Tag of message queue to pop from",
            content_type = "application/json"
        ),
        responses(
            (status = 200, description = "All message successfully extracted.", body = MessagePopAllResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Messages"
    )]
    pub async fn pop_all(mut req: Request<InternalState>) -> tide::Result<Response> {
        let tag: TagQueryRequest = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        let messages: Vec<MessagePopResponse> = inbox
            .pop_all(tag.tag)
            .await
            .into_iter()
            .filter_map(|(data, ts)| to_api_message(data, ts).ok())
            .collect::<Vec<_>>();

        Ok(Response::builder(200)
            .body(json!(MessagePopAllResponse { messages }))
            .build())
    }

    /// Peek the oldest message currently present in the nodes message inbox.
    ///
    /// The message is not removed from the inbox.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/messages/peek"),
        request_body(
            content = TagQueryRequest,
            description = "Tag of message queue to peek from",
            content_type = "application/json"
        ),
        responses(
            (status = 200, description = "Message successfully peeked at.", body = MessagePopResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Messages"
    )]
    pub async fn peek(mut req: Request<InternalState>) -> tide::Result<Response> {
        let tag: TagQueryRequest = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        if let Some((data, ts)) = inbox.peek(tag.tag).await {
            match to_api_message(data, ts) {
                Ok(message) => Ok(Response::builder(200).body(json!(message)).build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::UnknownFailure(e)).build()),
            }
        } else {
            Ok(Response::builder(404).build())
        }
    }

    /// Peek the list of messages currently present in the nodes message inbox, filtered by tag,
    /// and optionally by timestamp (epoch in milliseconds).
    /// The messages are not removed from the inbox.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/messages/peek-all"),
        request_body(
            content = GetMessageBodyRequest,
            description = "Tag of message queue and optionally a timestamp since from to start peeking",
            content_type = "application/json"
        ),
        responses(
            (status = 200, description = "All messages successfully peeked at.", body = MessagePopAllResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Messages"
    )]

    pub async fn peek_all(mut req: Request<InternalState>) -> tide::Result<Response> {
        let args: GetMessageBodyRequest = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        let messages = inbox
            .peek_all(args.tag, args.timestamp)
            .await
            .into_iter()
            .filter_map(|(data, ts)| to_api_message(data, ts).ok())
            .collect::<Vec<_>>();

        Ok(Response::builder(200)
            .body(json!(MessagePopAllResponse { messages }))
            .build())
    }
}

mod network {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct TicketPriceResponse {
        pub price: String,
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/network/price"),
        responses(
            (status = 200, description = "Current ticket price", body = TicketPriceResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Network"
    )]
    pub(super) async fn price(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match hopr.get_ticket_price().await {
            Ok(Some(price)) => Ok(Response::builder(200)
                .body(json!(TicketPriceResponse {
                    price: price.to_string()
                }))
                .build()),
            Ok(None) => Ok(Response::builder(422)
                .body(ApiErrorStatus::UnknownFailure(
                    "The ticket price is not available".into(),
                ))
                .build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }
}
mod tickets {
    use super::*;
    use hopr_crypto_types::types::Hash;
    use hopr_lib::{HoprTransportError, ProtocolError, Ticket, TicketStatistics, ToHex};

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "amount": "100",
        "channelEpoch": 1,
        "channelId": "0x04efc1481d3f106b88527b3844ba40042b823218a9cd29d1aa11c2c2ef8f538f",
        "index": 0,
        "indexOffset": 1,
        "signature": "0xe445fcf4e90d25fe3c9199ccfaff85e23ecce8773304d85e7120f1f38787f2329822470487a37f1b5408c8c0b73e874ee9f7594a632713b6096e616857999891",
        "winProb": "1"
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct ChannelTicket {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub channel_id: Hash,
        pub amount: String,
        pub index: u64,
        pub index_offset: u32,
        pub win_prob: String,
        pub channel_epoch: u32,
        pub signature: String,
    }

    impl From<Ticket> for ChannelTicket {
        fn from(value: Ticket) -> Self {
            Self {
                channel_id: value.channel_id,
                amount: value.amount.amount().to_string(),
                index: value.index,
                index_offset: value.index_offset,
                win_prob: value.win_prob().to_string(),
                channel_epoch: value.channel_epoch,
                signature: value.signature.expect("impossible to have an unsigned ticket").to_hex(),
            }
        }
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 200, description = "Channel funded successfully", body = [ChannelTicket]),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels"
    )]
    pub(super) async fn show_channel_tickets(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.tickets_in_channel(&channel_id).await {
                Ok(Some(tickets)) => Ok(Response::builder(200)
                    .body(json!(tickets
                        .into_iter()
                        .map(|t| ChannelTicket::from(t.ticket))
                        .collect::<Vec<_>>()))
                    .build()),
                Ok(None) => Ok(Response::builder(404).body(ApiErrorStatus::ChannelNotFound).build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidChannelId).build()),
        }
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/tickets"),
        responses(
            (status = 200, description = "Channel funded successfully", body = [ChannelTicket]),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Tickets"
    )]
    pub(super) async fn show_all_tickets(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match hopr.all_tickets().await {
            Ok(tickets) => Ok(Response::builder(200)
                .body(json!(tickets.into_iter().map(ChannelTicket::from).collect::<Vec<_>>()))
                .build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }

    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "losingTickets": 0,
        "neglected": 0,
        "neglectedValue": "0",
        "redeemed": 1,
        "redeemedValue": "100",
        "rejected": 0,
        "rejectedValue": "0",
        "unredeemed": 2,
        "unredeemedValue": "200",
        "winProportion": 1
    }))]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct NodeTicketStatisticsResponse {
        pub win_proportion: f64,
        pub unredeemed: u64,
        pub unredeemed_value: String,
        pub redeemed: u64,
        pub redeemed_value: String,
        pub losing_tickets: u64,
        pub neglected: u64,
        pub neglected_value: String,
        pub rejected: u64,
        pub rejected_value: String,
    }

    impl From<TicketStatistics> for NodeTicketStatisticsResponse {
        fn from(value: TicketStatistics) -> Self {
            Self {
                win_proportion: value.win_proportion,
                unredeemed: value.unredeemed,
                unredeemed_value: value.unredeemed_value.amount().to_string(),
                redeemed: value.redeemed,
                redeemed_value: value.redeemed_value.amount().to_string(),
                losing_tickets: value.losing,
                neglected: value.neglected,
                neglected_value: value.neglected_value.amount().to_string(),
                rejected: value.rejected,
                rejected_value: value.rejected_value.amount().to_string(),
            }
        }
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/tickets/statistics"),
        responses(
            (status = 200, description = "Tickets statistics fetched successfully. Check schema for description of every field in the statistics.", body = NodeTicketStatisticsResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Tickets"
    )]
    pub(super) async fn show_ticket_statistics(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match hopr.ticket_statistics().await.map(NodeTicketStatisticsResponse::from) {
            Ok(stats) => Ok(Response::builder(200).body(json!(stats)).build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/tickets/redeem"),
        responses(
            (status = 204, description = "Tickets redeemed successfully."),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Tickets"
    )]
    pub(super) async fn redeem_all_tickets(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match hopr.redeem_all_tickets(false).await {
            Ok(()) => Ok(Response::builder(204).build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets/redeem"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 204, description = "Tickets redeemed successfully."),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Tickets were not found for that channel. That means that no messages were sent inside this channel yet.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels"
    )]
    pub(super) async fn redeem_tickets_in_channel(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.redeem_tickets_in_channel(&channel_id, false).await {
                Ok(count) if count > 0 => Ok(Response::builder(204).build()),
                Ok(_) => Ok(Response::builder(404).body(ApiErrorStatus::TicketsNotFound).build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidChannelId).build()),
        }
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets/aggregate"),
        params(
            ("channelId" = String, Path, description = "ID of the channel.")
        ),
        responses(
            (status = 204, description = "Tickets successfully aggregated"),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Tickets were not found for that channel. That means that no messages were sent inside this channel yet.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Channels"
    )]
    pub(super) async fn aggregate_tickets_in_channel(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.aggregate_tickets(&channel_id).await {
                Ok(_) => Ok(Response::builder(204).build()),
                Err(HoprLibError::TransportError(HoprTransportError::Protocol(ProtocolError::ChannelNotFound))) => {
                    Ok(Response::builder(422).body(ApiErrorStatus::ChannelNotFound).build())
                }
                Err(HoprLibError::TransportError(HoprTransportError::Protocol(ProtocolError::ChannelClosed))) => {
                    Ok(Response::builder(422).body(ApiErrorStatus::ChannelNotOpen).build())
                }
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidChannelId).build()),
        }
    }
}

mod node {
    use super::*;
    use futures::StreamExt;
    use hopr_lib::{Health, Multiaddr};

    use {std::str::FromStr, tide::Body};

    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "version": "2.1.0"
    }))]
    pub(crate) struct NodeVersionResponse {
        pub version: String,
    }

    /// Get release version of the running node.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/node/version"),
        responses(
            (status = 200, description = "Fetched node version", body = NodeVersionResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Node"
    )]
    pub(super) async fn version(req: Request<InternalState>) -> tide::Result<Response> {
        let version = req.state().hopr.version();

        Ok(Response::builder(200)
            .body(json!(NodeVersionResponse { version }))
            .build())
    }

    #[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
    #[into_params(parameter_in = Query)]
    pub(crate) struct NodePeersQueryRequest {
        #[schema(required = false)]
        pub quality: Option<f64>,
    }

    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct HeartbeatInfo {
        pub sent: u64,
        pub success: u64,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct PeerInfo {
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        pub peer_id: PeerId,
        #[serde_as(as = "Option<DisplayFromStr>")]
        #[schema(value_type = Option<String>)]
        pub peer_address: Option<Address>,
        #[serde_as(as = "Option<DisplayFromStr>")]
        #[schema(value_type = Option<String>)]
        pub multiaddr: Option<Multiaddr>,
        pub heartbeats: HeartbeatInfo,
        pub last_seen: u128,
        pub last_seen_latency: u128,
        pub quality: f64,
        pub backoff: f64,
        pub is_new: bool,
        pub reported_version: String,
    }

    #[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct NodePeersResponse {
        pub connected: Vec<PeerInfo>,
        pub announced: Vec<PeerInfo>,
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
            ("api_token" = [])
        ),
        tag = "Node"
    )]
    pub(super) async fn peers(req: Request<InternalState>) -> tide::Result<Response> {
        let query_params: NodePeersQueryRequest = req.query()?;

        if let Some(quality) = query_params.quality {
            if !(0.0f64..=1.0f64).contains(&quality) {
                return Ok(Response::builder(400).body(ApiErrorStatus::InvalidQuality).build());
            }
        }

        let hopr = req.state().hopr.clone();

        let quality = query_params.quality.unwrap_or(0f64);
        let all_network_peers = futures::stream::iter(hopr.network_connected_peers().await)
            .filter_map(|peer| {
                let hopr = hopr.clone();

                async move {
                    if let Some(info) = hopr.network_peer_info(&peer).await {
                        if info.get_average_quality() >= quality {
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

                    // WARNING: Only in Providence are all peers public
                    let multiaddresses = hopr.multiaddresses_announced_to_dht(&peer_id).await;

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
                last_seen: info.last_seen as u128,
                last_seen_latency: info.last_seen_latency as u128,
                quality: info.get_average_quality(),
                backoff: info.backoff,
                is_new: info.heartbeats_sent == 0u64,
                reported_version: info
                    .metadata()
                    .get(&"protocol_version".to_owned())
                    .cloned()
                    .unwrap_or("UNKNOWN".to_string()),
            })
            .collect::<Vec<_>>()
            .await;

        let body = NodePeersResponse {
            connected: all_network_peers.clone(),
            announced: all_network_peers, // TODO: currently these are the same, since everybody has to announce
        };

        Ok(Response::builder(200).body(json!(body)).build())
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
        path = const_format::formatcp!("{BASE_PATH}/node/metrics"),
        responses(
            (status = 200, description = "Fetched node metrics", body = String),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = [])
        ),
        tag = "Node"
    )]
    pub(crate) async fn metrics(_req: Request<InternalState>) -> tide::Result<Response> {
        match collect_hopr_metrics() {
            Ok(metrics) => Ok(Response::builder(200)
                .body(Body::from_string(metrics))
                .content_type(Mime::from_str("text/plain; version=0.0.4").expect("must set mime type"))
                .build()),
            Err(error) => Ok(Response::builder(422).body(ApiErrorStatus::from(error)).build()),
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    #[schema(example = json!({
        "announcedAddress": [
            "/ip4/10.0.2.100/tcp/19092"
        ],
        "chain": "anvil-localhost",
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
        "network": "anvil-localhost"
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
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        hopr_token: Address,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        hopr_channels: Address,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        hopr_network_registry: Address,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        hopr_node_safe_registry: Address,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        hopr_management_module: Address,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        hopr_node_safe: Address,
        is_eligible: bool,
        #[serde_as(as = "DisplayFromStr")]
        #[schema(value_type = String)]
        connectivity_status: Health,
        /// Channel closure period in seconds
        channel_closure_period: u64,
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
            ("api_token" = [])
        ),
        tag = "Node"
    )]
    pub(super) async fn info(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let chain_config = hopr.chain_config();
        let safe_config = hopr.get_safe_config();
        let network = hopr.network();

        match hopr.get_channel_closure_notice_period().await {
            Ok(channel_closure_notice_period) => {
                let body = NodeInfoResponse {
                    network,
                    announced_address: hopr.local_multiaddresses(),
                    listening_address: hopr.local_multiaddresses(),
                    chain: chain_config.id,
                    hopr_token: chain_config.token,
                    hopr_channels: chain_config.channels,
                    hopr_network_registry: chain_config.network_registry,
                    hopr_node_safe_registry: chain_config.node_safe_registry,
                    hopr_management_module: chain_config.module_implementation,
                    hopr_node_safe: safe_config.safe_address,
                    is_eligible: hopr.is_allowed_to_access_network(&hopr.me_peer_id()).await,
                    connectivity_status: hopr.network_health().await,
                    channel_closure_period: channel_closure_notice_period.as_secs(),
                };

                Ok(Response::builder(200).body(json!(body)).build())
            }
            Err(error) => Ok(Response::builder(422).body(ApiErrorStatus::from(error)).build()),
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct EntryNode {
        #[serde_as(as = "Vec<DisplayFromStr>")]
        #[schema(value_type = Vec<String>)]
        pub multiaddrs: Vec<Multiaddr>,
        pub is_elligible: bool,
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
            ("api_token" = [])
        ),
        tag = "Node"
    )]
    pub(super) async fn entry_nodes(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match hopr.get_public_nodes().await {
            Ok(nodes) => {
                let mut body = HashMap::new();
                for (peer_id, address, mas) in nodes.into_iter() {
                    body.insert(
                        address.to_string(),
                        EntryNode {
                            multiaddrs: mas,
                            is_elligible: hopr.is_allowed_to_access_network(&peer_id).await,
                        },
                    );
                }

                Ok(Response::builder(200).body(json!(body)).build())
            }
            Err(error) => Ok(Response::builder(422).body(ApiErrorStatus::from(error)).build()),
        }
    }
}

mod checks {
    use super::*;

    /// Check whether the node is started.
    #[utoipa::path(
        get,
        path = "/startedz",
        responses(
            (status = 200, description = "The node is stared and running"),
            (status = 412, description = "The node is not started and running"),
        ),
        tag = "Checks"
    )]
    pub(super) async fn startedz(req: Request<State<'_>>) -> tide::Result<Response> {
        is_running(req).await
    }

    /// Check whether the node is ready to accept connections.
    #[utoipa::path(
        get,
        path = "/readyz",
        responses(
            (status = 200, description = "The node is ready to accept connections"),
            (status = 412, description = "The node is not ready to accept connections"),
        ),
        tag = "Checks"
    )]
    pub(super) async fn readyz(req: Request<State<'_>>) -> tide::Result<Response> {
        is_running(req).await
    }

    /// Check whether the node is healthy
    #[utoipa::path(
        get,
        path = "/healthyz",
        responses(
            (status = 200, description = "The node is healthy"),
            (status = 412, description = "The node is not healthy"),
        ),
        tag = "Checks"
    )]
    pub(super) async fn healthyz(req: Request<State<'_>>) -> tide::Result<Response> {
        is_running(req).await
    }

    async fn is_running(req: Request<State<'_>>) -> tide::Result<Response> {
        match req.state().hopr.status() {
            hopr_lib::HoprState::Running => Ok(Response::builder(200).build()),
            _ => Ok(Response::builder(412).build()),
        }
    }
}
