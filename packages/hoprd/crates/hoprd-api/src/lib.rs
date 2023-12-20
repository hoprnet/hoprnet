pub mod config;

use std::{sync::Arc, collections::HashMap};
use std::error::Error;
use std::collections::HashMap;

use async_std::sync::RwLock;
use libp2p_identity::PeerId;
use serde_json::json;
use serde_with::{serde_as, DisplayFromStr};
use tide::utils::async_trait;
use tide::{Middleware, Next, StatusCode};
use tide::{http::Mime, Request, Response};
use tide::http::headers::AUTHORIZATION;
use tide::http::mime;
use utoipa::OpenApi;
use utoipa_swagger_ui::Config;

use hopr_lib::errors::HoprLibError;
use hopr_lib::{Address, Balance, BalanceType, Hopr};

pub const BASE_PATH: &str = "/api/v3";
pub const API_VERSION: &str = "3.0.0";

#[derive(Clone)]
pub struct State<'a> {
    pub hopr: Arc<Hopr>,            // checks
    pub config: Arc<Config<'a>>,    // swagger
}

#[derive(Clone)]
pub struct InternalState {
    pub auth: Arc<crate::config::Auth>,
    pub hopr: Arc<Hopr>,
    pub inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    pub aliases: Arc<RwLock<HashMap<String, PeerId>>>,
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
        peers::show_all_peers,
        peers::ping_peer
    ),
    components(
        // schemas(todo::Todo, todo::TodoError)
    ),
    // modifiers(&SecurityAddon),
    tags(
        (name = "Check", description = "HOPR node functionality checks"),
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


/// Token-based authentication middleware
struct TokenBasedAuthenticationMiddleware {}

/// Implementation of the middleware
#[async_trait]
impl Middleware<InternalState> for TokenBasedAuthenticationMiddleware
{
    async fn handle(&self, request: Request<InternalState>, next: Next<'_, InternalState>) -> tide::Result {
        let auth = request.state().auth.clone();

        match auth.as_ref() {
            config::Auth::None => {},
            config::Auth::Token(token) => {
                let is_authorized = request.header(AUTHORIZATION)
                    .map(|auth| { token.as_str() == auth.as_str() })
                    .unwrap_or(false);

                if !is_authorized {
                    let reject_response = Response::builder(StatusCode::Unauthorized)
                        .content_type(mime::JSON)
                        .body(ApiErrorStatus::Unauthorized)
                        .build();

                    return Ok(reject_response);
                }
            },
        }

        // Go forward to the next middleware or request handler
        Ok(next.run(request).await)
    }
}

async fn serve_swagger(request: tide::Request<State<'_>>) -> tide::Result<Response> {
    let config = request.state().config.clone();
    let path = request.url().path().to_string();
    let tail = path.strip_prefix(&"swagger-ui/").unwrap();

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

pub async fn run_hopr_api(host: &str, cfg: &crate::config::Api, hopr: hopr_lib::Hopr, inbox: Arc<RwLock<hoprd_inbox::Inbox>>) {
    // Prepare alias part of the state
    let aliases: Arc<RwLock<HashMap<String, PeerId>>> = Arc::new(RwLock::new(HashMap::new()));
    aliases.write().await.insert("me".to_owned(), hopr.me_peer_id());

    let state = State {
        hopr: Arc::new(hopr),

        config: Arc::new(Config::from("openapi.json")),
    };

    let mut app = tide::with_state(state.clone());

    app.at("api-docs/openapi.json")
        .get(|_| async move { Ok(Response::builder(200).body(json!(ApiDoc::openapi()))) });

    app.at("swagger-ui/*").get(serve_swagger);

    app.at("startedz/").get(checks::startedz);
    app.at("readyz/").get(checks::readyz);
    app.at("healthyz/").get(checks::healthyz);

    app.at(&format!("{BASE_PATH}")).nest({
        let mut api = tide::with_state(InternalState {
            auth: Arc::new(cfg.auth.clone()),
            hopr: state.hopr.clone(),
            inbox,
            aliases,
        });

        api.with(TokenBasedAuthenticationMiddleware{});

        api.at("/aliases").get(alias::aliases).post(alias::set_alias);
        api.at("/aliases/:alias")
            .get(alias::get_alias)
            .delete(alias::delete_alias);

        api.at("/account/addresses").get(account::addresses);
        api.at("/account/balances").get(account::balances);
        api.at("/account/withdraw").get(account::withdraw);

        api.at("/peers/:peerId")
            .get(peers::show_all_peers)
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

        api.at("/messages/")
            .post(messages::send_message)
            .delete(messages::delete_messages);
        api.at("/messages/pop").get(messages::pop);
        api.at("/messages/pop-all").get(messages::pop_all);
        api.at("/messages/peek").get(messages::peek);
        api.at("/messages/peek-all").get(messages::peek_all);
        api.at("/messages/size").get(messages::size);

        api.at("/node/version").get(node::version);
        api.at("/node/info").get(node::info);
        api.at("/node/peers").get(node::peers);
        api.at("/node/metrics").get(node::metrics);
        api.at("/node/entryNodes").get(node::entry_nodes);

        api
    });

    app.listen(host).await.expect("the server should run successfully")
}

/// Should not be instantiated directly, but rather through the `ApiErrorStatus`.
#[derive(Debug, Clone, serde::Serialize)]
struct ApiError {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Enumerates all API request errors
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
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct PeerIdArg {
        #[serde_as(as = "DisplayFromStr")]
        pub peer_id: PeerId,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct AliasPeerId {
        pub alias: String,
        #[serde_as(as = "DisplayFromStr")]
        pub peer_id: PeerId,
    }

    /// Get each previously set alias and its corresponding PeerId
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/aliases/", BASE_PATH),
        responses(
            (status = 200, description = "Each alias with its corresponding PeerId", body = [AliasPeerId]),
        ),
        tag = "Alias"
    )]
    pub async fn aliases(req: Request<InternalState>) -> tide::Result<Response> {
        let aliases = req.state().aliases.clone();

        let aliases = aliases
            .read()
            .await
            .iter()
            .map(|(key, value)| AliasPeerId {
                alias: key.clone(),
                peer_id: value.clone(),
            })
            .collect::<Vec<_>>();

        Ok(Response::builder(200).body(json!(aliases)).build())
    }

    /// Set alias for a peer with a specific PeerId.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/aliases/", BASE_PATH),
        responses(
            (status = 201, description = "Alias set successfully.", body = PeerIdArg),
            (status = 400, description = "Invalid PeerId: The format or length of the peerId is incorrect.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Alias"
    )]
    pub async fn set_alias(mut req: Request<InternalState>) -> tide::Result<Response> {
        let args: AliasPeerId = req.body_json().await?;
        let aliases = req.state().aliases.clone();

        aliases.write().await.insert(args.alias, args.peer_id);
        Ok(Response::builder(200)
            .body(json!(PeerIdArg { peer_id: args.peer_id }))
            .build())
    }

    /// Get alias for the PeerId (Hopr address) that have this alias assigned to it.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/aliases/:alias", BASE_PATH),
        responses(
            (status = 200, description = "Get PeerId for an alias", body = int),
            (status = 404, description = "PeerId not found", body = ErrorNotFound),
        ),
        tag = "Alias"
    )]
    pub async fn get_alias(req: Request<InternalState>) -> tide::Result<Response> {
        let alias = req.param("alias")?.parse::<String>()?;
        let aliases = req.state().aliases.clone();

        let aliases = aliases.read().await;
        if let Some(peer_id) = aliases.get(&alias) {
            Ok(Response::builder(200)
                .body(json!(PeerIdArg {
                    peer_id: peer_id.clone()
                }))
                .build())
        } else {
            Ok(Response::builder(404).body(ApiErrorStatus::InvalidInput).build())
        }
    }

    /// Delete an alias.
    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{}/aliases/:alias", BASE_PATH),
        responses(
            (status = 204, description = "Alias removed successfully", body = int),
            (status = 422, description = "Unknown failure", body = ApiError)   // TOOD: This can never happen
        ),
        tag = "Alias"
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

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AccountAddresses {
        pub native: String,
        pub hopr: String,
    }

    /// Get node's HOPR and native addresses.
    ///
    /// HOPR address is represented by the P2P PeerId and can be used by other node owner to interact with this node.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/account/addresses", BASE_PATH),
        responses(
            (status = 200, description = "The node's public addresses", body = AddressesAddress),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Account"
    )]
    pub(super) async fn addresses(req: Request<InternalState>) -> tide::Result<Response> {
        let addresses = AccountAddresses {
            native: req.state().hopr.me_onchain().to_string(),
            hopr: req.state().hopr.me_peer_id().to_string(),
        };

        Ok(Response::builder(200).body(json!(addresses)).build())
    }

    #[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AccountBalances {
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
        path = const_format::formatcp!("{}/account/balances", BASE_PATH),
        responses(
            (status = 200, description = "The node's HOPR and Safe balances", body = AccountBalances),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Account"
    )]
    pub(super) async fn balances(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let mut account_balances = AccountBalances::default();

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

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WithdrawRequest {
        currency: BalanceType,
        amount: u128,
        // TODO: add validations here
        address: String,
    }

    /// Withdraw funds from this node to the ethereum wallet address.
    ///
    /// Both NATIVE or HOPR can be withdrawn using this method.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/account/withdraw", BASE_PATH),
        responses(
            (status = 200, description = "The node's funds have been withdrawn", body = AccountBalances),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Account"
    )]
    pub(super) async fn withdraw(mut req: Request<InternalState>) -> tide::Result<Response> {
        let withdraw_req_data: WithdrawRequest = req.body_json().await?;
        let recipient = <Address as std::str::FromStr>::from_str(&withdraw_req_data.address)?;

        match req
            .state()
            .hopr
            .withdraw(
                recipient,
                Balance::new(withdraw_req_data.amount.into(), withdraw_req_data.currency),
            )
            .await
        {
            Ok(receipt) => Ok(Response::builder(200).body(json!({"receipt": receipt})).build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }
}

mod peers {
    use std::str::FromStr;
    use std::time::Duration;
    use core_transport::constants::PEER_METADATA_PROTOCOL_VERSION;
    use hopr_lib::Multiaddr;
    use serde_with::DurationMilliSeconds;
    use core_transport::errors::HoprTransportError;
    use super::*;

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize)]
    struct NodePeerInfo {
        #[serde_as(as = "Vec<DisplayFromStr>")]
        pub announced: Vec<Multiaddr>,
        #[serde_as(as = "Vec<DisplayFromStr>")]
        pub observed: Vec<Multiaddr>
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/peers/{{peerId}}", BASE_PATH),
        responses(
            (status = 200, description = "Peer information fetched successfully.", body = NodePeerInfo),
            (status = 400, description = "Invalid peer id", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Peers"
    )]
    pub(super) async fn show_all_peers(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match PeerId::from_str(req.param("peerId")?) {
            Ok(peer) => Ok(
                Response::builder(200).body(json!(NodePeerInfo {
                    announced: hopr.multiaddresses_announced_to_dht(&peer).await,
                    observed: hopr.network_observed_multiaddresses(&peer).await
                }))
                .build()
            ),
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidPeerId).build())
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PingInfo {
        #[serde_as(as = "DurationMilliSeconds<u64>")]
        pub latency: std::time::Duration,
        pub reported_version: String
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/peers/{{peerId}}", BASE_PATH),
        responses(
            (status = 200, description = "Ping successful", body = NodePeerInfo),
            (status = 400, description = "Invalid peer id", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Peers"
    )]
    pub(super) async fn ping_peer(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match PeerId::from_str(req.param("peerId")?) {
            Ok(peer) => match hopr.ping(&peer).await {
                Ok(latency) => Ok(Response::builder(200)
                        .body(json!(PingInfo {
                            latency: latency.unwrap_or(Duration::ZERO), // TODO: what should be the correct default ?
                            reported_version: hopr.network_peer_info(&peer)
                                .await
                                .and_then(|s| s.metadata().get(PEER_METADATA_PROTOCOL_VERSION).cloned())
                                .unwrap_or("unknown".into())
                        }))
                        .build()
                ),
                Err(HoprLibError::TransportError(HoprTransportError::Protocol(core_protocol::errors::ProtocolError::Timeout))) =>
                    Ok(Response::builder(422).body(ApiErrorStatus::Timeout).build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build())
            },
            Err(_) => Ok(Response::builder(400).body(ApiErrorStatus::InvalidPeerId).build())
        }
    }
}

mod channels {
    use super::*;
    use core_crypto::types::Hash;
    use core_ethereum_actions::errors::CoreEthereumActionsError;
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use futures::TryFutureExt;
    use std::str::FromStr;
    use utils_types::traits::ToHex;

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct NodeChannel {
        #[serde_as(as = "DisplayFromStr")]
        pub id: Hash,
        #[serde_as(as = "DisplayFromStr")]
        pub peer_address: Address,
        #[serde_as(as = "DisplayFromStr")]
        pub status: ChannelStatus,
        pub balance: String,
    }

    impl From<ChannelEntry> for NodeChannel {
        fn from(value: ChannelEntry) -> Self {
            Self {
                id: value.get_id(),
                peer_address: value.destination,
                status: value.status,
                balance: value.balance.amount().to_string(),
            }
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct NodeTopologyChannel {
        #[serde_as(as = "DisplayFromStr")]
        pub channel_id: Hash,
        #[serde_as(as = "DisplayFromStr")]
        pub source_address: Address,
        #[serde_as(as = "DisplayFromStr")]
        pub destination_address: Address,
        #[serde_as(as = "DisplayFromStr")]
        pub source_peer_id: PeerId,
        #[serde_as(as = "DisplayFromStr")]
        pub destination_peer_id: PeerId,
        pub balance: String,
        #[serde_as(as = "DisplayFromStr")]
        pub status: ChannelStatus,
        pub ticket_index: u32,
        pub channel_epoch: u32,
        pub closure_time: u64,
    }

    #[derive(Debug, Clone, serde::Serialize)]
    struct NodeChannels {
        pub incoming: Vec<NodeChannel>,
        pub outgoing: Vec<NodeChannel>,
        pub all: Vec<NodeTopologyChannel>,
    }

    async fn query_topology_info(channel: &ChannelEntry, node: &Hopr) -> Result<NodeTopologyChannel, HoprLibError> {
        Ok(NodeTopologyChannel {
            channel_id: channel.get_id(),
            source_address: channel.source,
            destination_address: channel.destination,
            source_peer_id: node
                .chain_key_to_peerid(&channel.source)
                .await?
                .ok_or(HoprLibError::GeneralError("failed to map to peerid".into()))?,
            destination_peer_id: node
                .chain_key_to_peerid(&channel.destination)
                .await?
                .ok_or(HoprLibError::GeneralError("failed to map to peerid".into()))?,
            balance: channel.balance.amount().to_string(),
            status: channel.status,
            ticket_index: channel.ticket_index.as_u32(),
            channel_epoch: channel.channel_epoch.as_u32(),
            closure_time: channel.closure_time.as_u64(),
        })
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/channels", BASE_PATH),
        responses(
            (status = 200, description = "Channels fetched successfully", body = NodeChannels),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Channels"
    )]
    pub(super) async fn list_channels(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        let including_closed = bool::from_str(req.param("includingClosed")?)?;
        let full_topology = bool::from_str(req.param("fullTopology")?)?;

        if full_topology {
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
                    .body(json!(NodeChannels {
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
                    let channel_info = NodeChannels {
                        incoming: incoming
                            .into_iter()
                            .filter_map(|c| {
                                (including_closed || c.status != ChannelStatus::Closed).then(|| NodeChannel::from(c))
                            })
                            .collect(),
                        outgoing: outgoing
                            .into_iter()
                            .filter_map(|c| {
                                (including_closed || c.status != ChannelStatus::Closed).then(|| NodeChannel::from(c))
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
    #[derive(Debug, Clone, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OpenChannelRequest {
        #[serde_as(as = "DisplayFromStr")]
        pub peer_address: Address,
        pub amount: String,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct OpenChannelReceipt {
        #[serde_as(as = "DisplayFromStr")]
        pub channel_id: Hash,
        #[serde_as(as = "DisplayFromStr")]
        pub transaction_receipt: Hash,
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/channels", BASE_PATH),
        responses(
            (status = 201, description = "Channel successfully opened", body = OpenChannelReceipt),
            (status = 403, description = "Failed to open the channel because of insufficient HOPR balance or allowance.", body = ApiError),
            (status = 409, description = "Failed to open the channel because the channel between this nodes already exists.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Channels"
    )]
    pub(super) async fn open_channel(mut req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let open_req: OpenChannelRequest = req.body_json().await?;

        match hopr
            .open_channel(
                &open_req.peer_address,
                &Balance::new_from_str(&open_req.amount, BalanceType::HOPR),
            )
            .await
        {
            Ok(channel_details) => Ok(Response::builder(201)
                .body(json!(OpenChannelReceipt {
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
        responses(
            (status = 201, description = "Channel fetched successfully", body = NodeTopologyChannel),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Channels"
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
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CloseChannelReceipt {
        #[serde_as(as = "DisplayFromStr")]
        pub receipt: Hash,
        #[serde_as(as = "DisplayFromStr")]
        pub channel_status: ChannelStatus,
    }

    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{}/channels/{{channelId}}", BASE_PATH),
        responses(
            (status = 200, description = "Channel closed successfully", body = CloseChannelReceipt),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Channels"
    )]
    pub(super) async fn close_channel(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.close_channel_by_id(channel_id, false).await {
                Ok(receipt) => Ok(Response::builder(200)
                    .body(json!(CloseChannelReceipt {
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

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/channels/{{channelId}}/fund", BASE_PATH),
        responses(
            (status = 200, description = "Channel funded successfully", body = String),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Channels"
    )]
    pub(super) async fn fund_channel(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        let amount = Balance::new_from_str(req.param("amount")?, BalanceType::HOPR);

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

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct Tag {
        pub tag: u16,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct Size {
        pub size: usize,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[serde(rename_all = "camelCase")]
    struct SendMessageReq {
        /// The message tag used to filter messages based on application
        pub tag: u16,
        /// Message to be transmitted over the network
        pub body: String,
        /// The recipient HOPR PeerId
        #[serde_as(as = "DisplayFromStr")]
        pub peer_id: PeerId,
        #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
        // #[validate(length(min=0, max=3))]        // TODO: issue in serde_as with validator -> no order is correct
        pub path: Option<Vec<PeerId>>,
        #[validate(range(min = 1, max = 3))]
        pub hops: Option<u16>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct SendMessageRes {
        pub challenge: HalfKeyChallenge,
        pub timestamp: u128,
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[serde(rename_all = "camelCase")]
    struct GetMessageReq {
        /// The message tag used to filter messages based on application
        pub tag: u16,
        /// Timestamp to filter messages received after this timestamp
        #[serde_as(as = "Option<DurationMilliSeconds<u64>>")]
        pub timestamp: Option<std::time::Duration>,
    }

    /// Send a message to another peer using a given path.
    ///
    /// The message can be sent either over a specified path or using a specified
    /// number of HOPS, if no path is given.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/messages/", BASE_PATH),
        responses(
            (status = 202, description = "The message was sent successfully, DOES NOT imply successful delivery.", body = SendMessageRes),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Messages"
    )]
    pub async fn send_message(mut req: Request<InternalState>) -> tide::Result<Response> {
        let args: SendMessageReq = req.body_json().await?;
        let hopr = req.state().hopr.clone();

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
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        match hopr
            .send_message(
                Box::from(args.body.as_ref()),
                args.peer_id,
                args.path,
                args.hops,
                Some(args.tag),
            )
            .await
        {
            Ok(challenge) => Ok(Response::builder(202)
                .body(json!(SendMessageRes { challenge, timestamp }))
                .build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }

    /// Delete messages from nodes message inbox.
    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{}/messages/", BASE_PATH),
        responses(
            (status = 204, description = "Messages successfully deleted."),
        ),
        tag = "Messages"
    )]
    pub async fn delete_messages(req: Request<InternalState>) -> tide::Result<Response> {
        let tag: Tag = req.query()?;
        let inbox = req.state().inbox.clone();

        inbox.write().await.pop_all(Some(tag.tag)).await;
        Ok(Response::builder(204).build())
    }

    /// Get size of filtered message inbox for a specific tag
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/messages/size/", BASE_PATH),
        responses(
            (status = 200, description = "Returns the message inbox size filtered by the given tag", body = Size),
        ),
        tag = "Messages"
    )]
    pub async fn size(req: Request<InternalState>) -> tide::Result<Response> {
        let tag: Tag = req.query()?;
        let inbox = req.state().inbox.clone();

        let size = inbox.read().await.size(Some(tag.tag)).await;

        Ok(Response::builder(200).body(json!(Size { size })).build())
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct MessagePopRes {
        tag: u16,
        body: String,
        #[serde_as(as = "serde_with::DurationMilliSeconds<u64>")]
        received_at: std::time::Duration,
    }

    fn to_api_message(data: hopr_lib::ApplicationData, ts: Duration) -> Result<MessagePopRes, String> {
        if let Some(tag) = data.application_tag {
            match std::str::from_utf8(&data.plain_text) {
                Ok(data_str) => Ok(MessagePopRes {
                    tag,
                    body: data_str.into(),
                    received_at: ts.as_millis(),
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
        path = const_format::formatcp!("{}/messages/pop", BASE_PATH),
        responses(
            (status = 204, description = "Message successfully extracted.", body = MessagePopRes),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Messages"
    )]
    pub async fn pop(mut req: Request<InternalState>) -> tide::Result<Response> {
        let tag: Tag = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        if let Some((data, ts)) = inbox.pop(Some(tag.tag)).await {
            match to_api_message(data, ts) {
                Ok(message) => Ok(Response::builder(204).body(json!(message)).build()),
                Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::UnknownFailure(e)).build()),
            }
        } else {
            Ok(Response::builder(404).build())
        }
    }

    /// Get the list of messages currently present in the nodes message inbox.
    ///
    /// The messages are removed from the inbox.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/messages/pop-all", BASE_PATH),
        responses(
            (status = 200, description = "All message successfully extracted.", body = [MessagePopRes]),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Messages"
    )]
    pub async fn pop_all(mut req: Request<InternalState>) -> tide::Result<Response> {
        let tag: Tag = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        let messages: Vec<MessagePopRes> = inbox
            .pop_all(Some(tag.tag))
            .await
            .into_iter()
            .filter_map(|(data, ts)| to_api_message(data, ts).ok())
            .collect::<Vec<_>>();

        Ok(Response::builder(200).body(json!(messages)).build())
    }

    /// Peek the oldest message currently present in the nodes message inbox.
    ///
    /// The message is not removed from the inbox.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/messages/peek", BASE_PATH),
        responses(
            (status = 204, description = "Message successfully peeked at.", body = MessagePopRes),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Messages"
    )]
    pub async fn peek(mut req: Request<InternalState>) -> tide::Result<Response> {
        let tag: Tag = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        if let Some((data, ts)) = inbox.pop(Some(tag.tag)).await {
            match to_api_message(data, ts) {
                Ok(message) => Ok(Response::builder(204).body(json!(message)).build()),
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
        path = const_format::formatcp!("{}/messages/peek-all", BASE_PATH),
        responses(
            (status = 200, description = "All messages successfully peeked at.", body = [MessagePopRes]),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Messages"
    )]

    pub async fn peek_all(mut req: Request<InternalState>) -> tide::Result<Response> {
        let args: GetMessageReq = req.body_json().await?;
        let ts: Option<Duration> = args.timestamp.map(|ts| Duration::from_millis(ts.try_into().unwrap()));
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        let messages = inbox
            .peek_all(Some(args.tag), ts)
            .await
            .into_iter()
            .filter_map(|(data, ts)| to_api_message(data, ts).ok())
            .collect::<Vec<_>>();

        Ok(Response::builder(200).body(json!(messages)).build())
    }
}

mod tickets {
    use super::*;
    use core_crypto::types::Hash;
    use core_protocol::errors::ProtocolError;
    use core_transport::errors::HoprTransportError;
    use core_transport::TicketStatistics;
    use core_types::channels::Ticket;
    use utils_types::traits::ToHex;

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ChannelTicket {
        #[serde_as(as = "DisplayFromStr")]
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
        path = const_format::formatcp!("{}/channels/{{channelId}}/tickets", BASE_PATH),
        responses(
            (status = 200, description = "Channel funded successfully", body = [ChannelTicket]),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 404, description = "Channel not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
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
        path = const_format::formatcp!("{}/tickets", BASE_PATH),
        responses(
            (status = 200, description = "Channel funded successfully", body = [ChannelTicket]),
            (status = 422, description = "Unknown failure", body = ApiError)
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

    #[derive(Debug, Clone, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    struct NodeTicketStatistics {
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

    impl From<TicketStatistics> for NodeTicketStatistics {
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
        path = const_format::formatcp!("{}/tickets/statistics", BASE_PATH),
        responses(
            (status = 200, description = "Tickets statistics fetched successfully. Check schema for description of every field in the statistics.", body = NodeTicketStatistics),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Tickets"
    )]
    pub(super) async fn show_ticket_statistics(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match hopr.ticket_statistics().await.map(NodeTicketStatistics::from) {
            Ok(stats) => Ok(Response::builder(200).body(json!(stats)).build()),
            Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::from(e)).build()),
        }
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/tickets/redeem", BASE_PATH),
        responses(
            (status = 200, description = "Tickets redeemed successfully."),
            (status = 422, description = "Unknown failure", body = ApiError)
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
        path = const_format::formatcp!("{}/channel/{{channelId}}/tickets/redeem", BASE_PATH),
        responses(
            (status = 200, description = "Tickets redeemed successfully."),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 404, description = "Tickets were not found for that channel. That means that no messages were sent inside this channel yet.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Channel"
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
        path = const_format::formatcp!("{}/channel/{{channelId}}/tickets/aggregate", BASE_PATH),
        responses(
            (status = 204, description = "Tickets successfully aggregated"),
            (status = 400, description = "Invalid channel id.", body = ApiError),
            (status = 404, description = "Tickets were not found for that channel. That means that no messages were sent inside this channel yet.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Channel"
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
    use std::str::FromStr;
    use hopr_lib::{Multiaddr, Health};
    use tide::Body;

    use super::*;

    /// Get release version of the running node.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/node/version", BASE_PATH),
        responses(
            (status = 200, description = "Fetched node version"),
        ),
        tag = "Node"
    )]
    pub(super) async fn version(req: Request<InternalState>) -> tide::Result<Response> {
        let version = req.state().hopr.version();

        Ok(Response::builder(200).body(json!({"version": version})).build())
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct NodePeersReqQuery {
        quality: Option<f64>
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct HeartbeatInfo {
        sent: u64,
        success: u64
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PeerInfo {
        #[serde_as(as = "DisplayFromStr")]
        peer_id: PeerId,
        #[serde_as(as = "Option<DisplayFromStr>")]
        peer_address: Option<Address>,
        #[serde_as(as = "Option<DisplayFromStr>")]
        multiaddr: Option<Multiaddr>,
        heartbeats: HeartbeatInfo,
        last_seen: u128,
        last_seen_latency: u128,
        quality: f64,
        backoff: f64,
        is_new: bool,
        reported_version: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct NodePeersRes {
        connected: Vec<PeerInfo>,
        announced: Vec<PeerInfo>
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
        path = const_format::formatcp!("{}/node/peers", BASE_PATH),
        responses(
            (status = 200, description = "Successfully returned observed peers", body=NodePeersRes),
            (status = 400, description = "Failed to extract a valid quality parameter", body = ApiError),
        ),
        tag = "Node"
    )]
    pub(super) async fn peers(req: Request<InternalState>) -> tide::Result<Response> {
        let query_params: NodePeersReqQuery = req.query()?;

        if let Some(quality) = query_params.quality {
            if quality < 0.0f64 || quality > 1.0f64 {
                return Ok(Response::builder(400).body(ApiErrorStatus::InvalidQuality).build());
            }
        }

        let hopr = req.state().hopr.clone();

        let body = NodePeersRes{
            connected: hopr.all_network_peers(query_params.quality.unwrap_or(0f64))
                            .await
                            .into_iter()
                            .map(|(address, peer_id, info)| {
                                PeerInfo {
                                    peer_id,
                                    peer_address: address,
                                    multiaddr: None,
                                    heartbeats: HeartbeatInfo {
                                        sent: info.heartbeats_sent,
                                        success: info.heartbeats_succeeded
                                    },
                                    last_seen: info.last_seen as u128,
                                    last_seen_latency: info.last_seen_latency as u128,
                                    quality: info.get_average_quality(),
                                    backoff: info.backoff,
                                    is_new: info.heartbeats_sent == 0u64,
                                    reported_version: "TODO: Add version here".into()
                                }
                            })
                            .collect(),
            announced: vec![]
        };

        Ok(Response::builder(200).body(json!(body)).build())
    }

    /// Retrieve Prometheus metrics from the running node.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/node/metrics", BASE_PATH),
        responses(
            (status = 200, description = "Fetched node metrics", body = String),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Node"
    )]
    pub(super) async fn metrics(_req: Request<InternalState>) -> tide::Result<Response> {
        match utils_metrics::metrics::gather_all_metrics() {
            Ok(metrics) => Ok(Response::builder(200)
                .body(Body::from_string(metrics))
                .content_type(Mime::from_str("text/plain; version=0.0.4").expect("must set mime type"))
                .build()
            ),
            Err(error) => Ok(Response::builder(422).body(ApiErrorStatus::from(error)).build()),
        }
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct NodeInfoRes {
        network: String,
        #[serde_as(as = "Vec<DisplayFromStr>")]
        announced_address: Vec<Multiaddr>,
        #[serde_as(as = "Vec<DisplayFromStr>")]
        listening_address: Vec<Multiaddr>,
        chain: String,
        #[serde_as(as = "DisplayFromStr")]
        hopr_token: Address,
        #[serde_as(as = "DisplayFromStr")]
        hopr_channels: Address,
        #[serde_as(as = "DisplayFromStr")]
        hopr_network_registry: Address,
        #[serde_as(as = "DisplayFromStr")]
        hopr_node_sage_registry: Address,
        #[serde_as(as = "DisplayFromStr")]
        hopr_management_module: Address,
        #[serde_as(as = "DisplayFromStr")]
        hopr_node_safe: Address,
        is_eligible: bool,
        #[serde_as(as = "DisplayFromStr")]
        connectivity_status: Health,
        channel_closure_period: u64
    }

    /// Get information about this HOPR Node.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/node/info", BASE_PATH),
        responses(
            (status = 200, description = "Fetched node version"),
        ),
        tag = "Node"
    )]
    pub(super) async fn info(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let chain_config = hopr.chain_config();
        let network = hopr.network();

        let body = NodeInfoRes {
            network,
            announced_address: hopr.local_multiaddresses(),
            listening_address: hopr.local_multiaddresses(),
            chain: chain_config.id,
            hopr_token: chain_config.token,
            hopr_channels: chain_config.channels,
            hopr_network_registry: chain_config.network_registry,
            hopr_node_sage_registry: chain_config.node_safe_registry,
            hopr_management_module: chain_config.module_implementation,
            hopr_node_safe: chain_config.node_safe_registry,    // TODO: bad value, what should be here?
            is_eligible: hopr.is_allowed_to_access_network(&hopr.me_peer_id()).await,
            connectivity_status: hopr.network_health().await,
            channel_closure_period: 0u64    // TODO: bad value, what should be here?
        };

        Ok(Response::builder(200).body(json!(body)).build())
    }

    #[serde_as]
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct EntryNode {
        #[serde_as(as = "Vec<DisplayFromStr>")]
        pub multiaddrs: Vec<Multiaddr>,
        pub is_elligible: bool
    }

    /// List all known entry nodes with multiaddrs and eligibility.
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{}/node/entryNodes", BASE_PATH),
        responses(
            (status = 200, description = "Fetched public nodes' information", body = HashMap<String, EntryNode>),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        tag = "Node"
    )]
    pub(super) async fn entry_nodes(req: Request<InternalState>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match hopr.get_public_nodes().await {
            Ok(nodes) => {
                let mut body = HashMap::new();
                for (peer_id, address, mas) in nodes.into_iter() {
                    body.insert(address, EntryNode {
                        multiaddrs: mas,
                        is_elligible: hopr.is_allowed_to_access_network(&peer_id).await
                    });
                }

                Ok(Response::builder(200).body(json!(body)).build())
            },
            Err(error) => Ok(Response::builder(422).body(ApiErrorStatus::from(error)).build()),
        }
    }
}

mod checks {
    use super::*;

    /// Check whether the node is started.
    #[utoipa::path(
        get,
        path = "startedz",
        responses(
            (status = 200, description = "The node is stared and running"),
            (status = 412, description = "The node is not started and running"),
        )
    )]
    pub(super) async fn startedz(req: Request<State<'_>>) -> tide::Result<Response> {
        is_running(req).await
    }

    /// Check whether the node is ready to accept connections.
    #[utoipa::path(
        get,
        path = "readyz",
        responses(
            (status = 200, description = "The node is ready to accept connections"),
            (status = 412, description = "The node is not ready to accept connections", ),
        )
    )]
    pub(super) async fn readyz(req: Request<State<'_>>) -> tide::Result<Response> {
        is_running(req).await
    }

    /// Check whether the node is healthy
    #[utoipa::path(
        get,
        path = "healthyz",
        responses(
            (status = 200, description = "The node is healthy"),
            (status = 412, description = "The node is not healthy"),
        )
    )]
    pub(super) async fn healthyz(req: Request<State<'_>>) -> tide::Result<Response> {
        is_running(req).await
    }

    async fn is_running(req: Request<State<'_>>) -> tide::Result<Response> {
        match req.state().hopr.status() {
            hopr_lib::State::Running => Ok(Response::builder(200).build()),
            _ => Ok(Response::builder(412).build()),
        }
    }
}
