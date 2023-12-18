use std::{sync::Arc, collections::HashMap};
use std::fmt::Display;

use async_std::sync::RwLock;
use libp2p_identity::PeerId;
use serde_json::json;
use tide::{http::Mime, Request, Response};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::Config;

use hopr_lib::{Address, Balance, BalanceType, Hopr};
use hopr_lib::errors::HoprLibError;

pub const BASE_PATH: &str = "/api/v3";
pub const API_VERSION: &str = "3.0.0";

#[derive(Clone)]
pub struct State<'a> {
    pub hopr: Arc<Hopr>,
    pub inbox: Arc<RwLock<hoprd_inbox::Inbox>>,
    pub aliases: Arc<RwLock<HashMap<String, PeerId>>>,
    pub config: Arc<Config<'a>>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        // todo::list_todos,
        // todo::create_todo,
        // todo::delete_todo,
        // todo::mark_done
    ),
    components(
        // schemas(todo::Todo, todo::TodoError)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "todo", description = "Todo items management endpoints.")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, _openapi: &mut utoipa::openapi::OpenApi) {
        // let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        // components.add_security_scheme(
        //     "api_key",
        //     SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
        // )
    }
}

async fn serve_swagger(request: tide::Request<State<'_>>) -> tide::Result<Response> {
    let config = request.state().config.clone();
    let path = request.url().path().to_string();
    let tail = path.strip_prefix(&format!("{BASE_PATH}/swagger-ui/")).unwrap();

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

pub async fn run_hopr_api(host: &str, hopr: hopr_lib::Hopr, inbox: Arc<RwLock<hoprd_inbox::Inbox>>) {
    // Prepare alias part of the state
    let aliases: Arc<RwLock<HashMap<String, PeerId>>> = Arc::new(RwLock::new(HashMap::new()));
    aliases
        .write()
        .await
        .insert("me".to_owned(), hopr.me_peer_id());

    let state = State {
        hopr: Arc::new(hopr),
        aliases,
        inbox,
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
        let mut api = tide::with_state(state);

        api.at("/aliases")
            .get(alias::aliases)
            .post(alias::set_alias);
        api.at("/aliases/:alias")
            .get(alias::get_alias)
            .delete(alias::delete_alias);

        api.at("/account/addresses").get(account::addresses);
        api.at("/account/balances").get(account::balances);
        api.at("/account/withdraw").get(account::withdraw);

        api.at("/channels")
            .get(channels::list_channels)
            .post(channels::open_channel);

        api.at("/channels/:channelId")
            .get(channels::show_channel)
            .delete(channels::close_channel);

        api.at("/channels/:channelId/fund")
            .post(channels::fund_channel);

        api.at("/channels/:channelId/tickets")
            .get(tickets::show_channel_tickets);

        api.at("/tickets")
            .get(tickets::show_all_tickets);

        api
    });

    app.listen(host).await.expect("the server should run successfully")
}

#[derive(
    Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate, utoipa::IntoResponses,
)]
#[response(status = 422)]
pub struct Error422 {
    pub status: String,
    pub error: String,
}

impl Error422 {
    pub fn new(error: String) -> Self {
        Self {
            status: "UNKNOWN_FAILURE".into(),
            error,
        }
    }
}

impl<T: Display> From<T> for Error422 {
    fn from(value: T) -> Self {
        Self::new(value.to_string())
    }
}

impl From<Error422> for tide::Body {
    fn from(value: Error422) -> Self {
        json!(value).into()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RequestStatus {
    pub status: String,
}

impl From<RequestStatus> for tide::Body {
    fn from(value: RequestStatus) -> Self {
        json!(value).into()
    }
}

mod alias {
    use super::*;
    use std::str::FromStr;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct PeerIdArg {
        pub peer_id: String,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct AliasPeerId {
        pub alias: String,
        /// HOPR node's offchain address representing a transport P2P address (PeerId)
        pub peer_id: String,
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
    pub async fn aliases(req: Request<State<'_>>) -> tide::Result<Response> {
        let aliases = req.state().aliases.clone();

        let aliases = aliases.read()
            .await
            .iter()
            .map(|(key, value)| {
                AliasPeerId {
                    alias: key.clone(),
                    peer_id: value.to_string(),
                }
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
            (status = 400, description = "Invalid PeerId: The format or length of the peerId is incorrect.", body = RequestStatus),
            Error422,
        ),
        tag = "Alias"
    )]
    pub async fn set_alias(mut req: Request<State<'_>>) -> tide::Result<Response> {
        let args: AliasPeerId = req.body_json().await?;
        let aliases = req.state().aliases.clone();

        match PeerId::from_str(&args.peer_id) {
            Ok(peer_id) => {
                aliases.write().await.insert(args.alias, peer_id.clone());
                Ok(Response::builder(200)
                    .body(json!(PeerIdArg{peer_id: peer_id.to_string()}))
                    .build()
                )
            },
            Err(error) => {
                Ok(Response::builder(400)
                    .body(json!(RequestStatus{status: format!("Invalid PeerId '{}': {error}", args.peer_id)}))
                    .build()
                )
            },
        }
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
    pub async fn get_alias(req: Request<State<'_>>) -> tide::Result<Response> {
        let alias = req.param("alias")?.parse::<String>()?;
        let aliases = req.state().aliases.clone();

        let aliases = aliases.read().await;
        if let Some(peer_id) = aliases.get(&alias) {
            Ok(Response::builder(200)
                .body(json!(PeerIdArg{peer_id: peer_id.to_string()}))
                .build()
            )
        } else {
            Ok(Response::builder(404)
                .body(json!(RequestStatus{status: format!("The alias '{alias}' does not exist")}))
                .build()
            )
        }
    }

    /// Delete an alias.
    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{}/aliases/:alias", BASE_PATH),
        responses(
            (status = 204, description = "Alias removed successfully", body = int),
            Error422,   // TOOD: This can never happen
        ),
        tag = "Alias"
    )]
    pub async fn delete_alias(req: Request<State<'_>>) -> tide::Result<Response> {
        let alias = req.param("alias")?.parse::<String>()?;
        let aliases = req.state().aliases.clone();

        let _ = aliases.write().await.remove(&alias);

        Ok(Response::builder(204).build())
    }

//     .get(alias::get_aliases)
//     .post(alias::post_aliases);
// api.at("/aliases/:alias")
//     .get(alias::set_alias)
//     .delete(alias::set_alias)
//     .post(alias::delete_alias);

    // pub async fn remove_alias(&self, alias: &String) {
    //     ;
    // }

    // pub async fn get_alias(&self, alias: &String) -> Option<PeerId> {
    //     self.aliases.read().await.get(alias).copied()
    // }

    // pub async fn get_aliases(&self) -> HashMap<String, PeerId> {
    //     self.aliases.read().await.clone()
    // }
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
        path = const_format::formatcp!("{BASE_PATH}/account/addresses"),
        responses(
            (status = 200, description = "The node's public addresses", body = AddressesAddress),
            Error422,
        ),
        tag = "Account"
    )]
    pub(super) async fn addresses(req: Request<State<'_>>) -> tide::Result<Response> {
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
        path = const_format::formatcp!("{BASE_PATH}/account/balances"),
        responses(
            (status = 200, description = "The node's HOPR and Safe balances", body = AccountBalances),
            Error422,
        ),
        tag = "Account"
    )]
    pub(super) async fn balances(req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let mut account_balances = AccountBalances::default();

        match hopr.get_balance(BalanceType::Native).await {
            Ok(v) => account_balances.native = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(Error422::new(e.to_string())).build()),
        }

        match hopr.get_balance(BalanceType::HOPR).await {
            Ok(v) => account_balances.hopr = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(Error422::new(e.to_string())).build()),
        }

        match hopr.get_safe_balance(BalanceType::Native).await {
            Ok(v) => account_balances.safe_native = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(Error422::new(e.to_string())).build()),
        }

        match hopr.get_safe_balance(BalanceType::HOPR).await {
            Ok(v) => account_balances.safe_hopr = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(Error422::new(e.to_string())).build()),
        }

        match hopr.safe_allowance().await {
            Ok(v) => account_balances.safe_hopr_allowance = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(Error422::new(e.to_string())).build()),
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
    /// Both NATIVE or HOPR can be withdrawn using this method."
    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/account/withdraw"),
        responses(
            (status = 200, description = "The node;s funds have been withdrawn", body = AccountBalances),
            Error422,
        ),
        tag = "Account"
    )]
    pub(super) async fn withdraw(mut req: Request<State<'_>>) -> tide::Result<Response> {
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
            Err(e) => Ok(Response::builder(422).body(Error422::new(e.to_string())).build()),
        }
    }
}

mod channels {
    use std::str::FromStr;
    use futures::TryFutureExt;
    use core_crypto::types::Hash;
    use core_ethereum_actions::errors::CoreEthereumActionsError;
    use core_types::channels::{ChannelEntry, ChannelStatus};
    use utils_types::traits::ToHex;
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct NodeChannel {
        id: Hash,
        peer_address: Address,
        status: ChannelStatus,
        balance: Balance
    }

    impl From<ChannelEntry> for NodeChannel {
        fn from(value: ChannelEntry) -> Self {
            Self {
                id: value.get_id(),
                peer_address: value.destination,
                status: value.status,
                balance: value.balance
            }
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct NodeTopologyChannel {
        channel_id: Hash,
        source_address: Address,
        destination_address: Address,
        source_peer_id: PeerId,
        destination_peer_id: PeerId,
        balance: Balance,
        status: ChannelStatus,
        ticket_index: u32,
        channel_epoch: u32,
        closure_time: u64
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct NodeChannels {
        incoming: Vec<NodeChannel>,
        outgoing: Vec<NodeChannel>,
        all: Vec<NodeTopologyChannel>
    }

    async fn query_topology_info(channel: &ChannelEntry, node: &Hopr) -> Result<NodeTopologyChannel, HoprLibError> {
        Ok(NodeTopologyChannel {
            channel_id: channel.get_id(),
            source_address: channel.source,
            destination_address: channel.destination,
            source_peer_id: node.chain_key_to_peerid(&channel.source).await?.ok_or(HoprLibError::GeneralError("failed to map to peerid".into()))?,
            destination_peer_id: node.chain_key_to_peerid(&channel.destination).await?.ok_or(HoprLibError::GeneralError("failed to map to peerid".into()))?,
            balance: channel.balance,
            status: channel.status,
            ticket_index: channel.ticket_index.as_u32(),
            channel_epoch: channel.channel_epoch.as_u32(),
            closure_time: channel.closure_time.as_u64()
        })
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        responses(
            (status = 200, description = "Channels fetched successfully", body = NodeChannels),
            Error422,
        ),
        tag = "Channels"
    )]
    pub(super) async fn list_channels(req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        let including_closed = bool::from_str(req.param("includingClosed")?)?;
        let full_topology = bool::from_str(req.param("fullTopology")?)?;

        if full_topology {
            let hopr_clone = hopr.clone();
            let topology = hopr.all_channels()
                .and_then(|channels| async move {
                    futures::future::try_join_all(
                        channels
                            .iter()
                            .map(|c| query_topology_info(c, hopr_clone.as_ref()))

                    ).await
                }
            )
            .await;

            match topology {
                Ok(all) => Ok(Response::builder(200).body(json!(NodeChannels { incoming: vec![], outgoing: vec![], all })).build()),
                Err(e) => Ok(Response::builder(422).body(Error422::new(e.to_string())).build())
            }
        } else {
            let channels = hopr.channels_to(&hopr.me_onchain())
                .and_then(|incoming| async {
                    let outgoing = hopr.channels_from(&hopr.me_onchain()).await?;
                    Ok((incoming, outgoing))
                })
                .await;

            match channels {
                Ok((incoming, outgoing)) => {
                    let channel_info = NodeChannels {
                        incoming: incoming.into_iter()
                            .filter_map(|c| (including_closed || c.status != ChannelStatus::Closed).then(|| NodeChannel::from(c)))
                            .collect(),
                        outgoing: outgoing.into_iter()
                            .filter_map(|c| (including_closed || c.status != ChannelStatus::Closed).then(|| NodeChannel::from(c)))
                            .collect(),
                        all: vec![],
                    };

                    Ok(Response::builder(200).body(json!(channel_info)).build())
                }
                Err(e) => Ok(Response::builder(422).body(Error422::new(e.to_string())).build())
            }
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OpenChannelRequest {
        peer_address: Address,
        amount: Balance
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct OpenChannelReceipt {
        channel_id: Hash,
        transaction_receipt: Hash
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels"),
        responses(
            (status = 201, description = "Channel successfully opened", body = OpenChannelReceipt),
            (status = 403, description = "Failed to open the channel because of insufficient HOPR balance or allowance.", body = RequestStatus),
            (status = 409, description = "Failed to open the channel because the channel between this nodes already exists.", body = RequestStatus),
            Error422,
        ),
        tag = "Channels"
    )]
    pub(super) async fn open_channel(mut req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        let open_req: OpenChannelRequest = req.body_json().await?;

        match hopr.open_channel(&open_req.peer_address, &open_req.amount).await {
            Ok(channel_details) => {
                Ok(Response::builder(201).body(json!(OpenChannelReceipt {
                    channel_id: channel_details.channel_id,
                    transaction_receipt: channel_details.tx_hash
                })).build())
            }
            Err(HoprLibError::ChainError(CoreEthereumActionsError::BalanceTooLow)) => {
                Ok(Response::builder(403).body(RequestStatus { status: "NOT_ENOUGH_BALANCE".into() }).build())
            },
            Err(HoprLibError::ChainError(CoreEthereumActionsError::NotEnoughAllowance)) => {
                Ok(Response::builder(403).body(RequestStatus { status: "NOT_ENOUGH_ALLOWANCE".into() }).build())
            },
            Err(HoprLibError::ChainError(CoreEthereumActionsError::ChannelAlreadyExists)) => {
                Ok(Response::builder(409).body(RequestStatus { status: "CHANNEL_ALREADY_OPEN".into() }).build())
            }
            Err(e) => {
                Ok(Response::builder(422).body(Error422::from(e)).build())
            }
        }
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}"),
        responses(
            (status = 201, description = "Channel fetched successfully", body = NodeTopologyChannel),
            (status = 400, description = "Invalid channel id.", body = RequestStatus),
            (status = 404, description = "Channel not found.", body = RequestStatus),
            Error422,
        ),
        tag = "Channels"
    )]
    pub(super) async fn show_channel(req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.channel_from_hash(&channel_id).await {
                Ok(Some(channel)) => Ok(Response::builder(200).body(json!(query_topology_info(&channel, hopr.as_ref()).await?)).build()),
                Ok(None) => Ok(Response::builder(404).body(RequestStatus { status: "CHANNEL_NOT_FOUND".into() }).build()),
                Err(e) => Ok(Response::builder(422).body(Error422::from(e)).build())
            },
            Err(_) => Ok(Response::builder(400).body(RequestStatus { status: "INVALID_CHANNELID".into() }).build())
        }
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CloseChannelReceipt {
        receipt: Hash,
        channel_status: ChannelStatus
    }

    #[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}"),
        responses(
            (status = 200, description = "Channel closed successfully", body = CloseChannelReceipt),
            (status = 400, description = "Invalid channel id.", body = RequestStatus),
            (status = 404, description = "Channel not found.", body = RequestStatus),
            Error422,
        ),
        tag = "Channels"
    )]
    pub(super) async fn close_channel(req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.close_channel_by_id(channel_id, false).await {
                Ok(receipt) => {
                    Ok(Response::builder(200).body(json!(CloseChannelReceipt {
                        channel_status: receipt.status,
                        receipt: receipt.tx_hash
                    })).build())
                },
                Err(HoprLibError::ChainError(CoreEthereumActionsError::ChannelDoesNotExist)) => {
                    Ok(Response::builder(404).body(RequestStatus { status: "CHANNEL_NOT_FOUND".into() }).build())
                },
                Err(HoprLibError::ChainError(CoreEthereumActionsError::InvalidArguments(_))) => {
                    Ok(Response::builder(422).body(RequestStatus { status: "UNSUPPORTED_FEATURE".into() }).build())
                },
                Err(e) => Ok(Response::builder(422).body(Error422::from(e)).build())
            },
            Err(_) => Ok(Response::builder(400).body(RequestStatus { status: "INVALID_CHANNELID".into() }).build())
        }
    }

    #[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/fund"),
        responses(
            (status = 200, description = "Channel funded successfully", body = String),
            (status = 400, description = "Invalid channel id.", body = RequestStatus),
            (status = 404, description = "Channel not found.", body = RequestStatus),
            Error422,
        ),
        tag = "Channels"
    )]
    pub(super) async fn fund_channel(req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        let amount = Balance::new_from_str(req.param("amount")?, BalanceType::HOPR);

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => {
                match hopr.fund_channel(&channel_id, &amount).await {
                    Ok(hash) => Ok(Response::builder(200).body(hash.to_string()).build()),
                    Err(HoprLibError::ChainError(CoreEthereumActionsError::ChannelDoesNotExist)) =>
                        Ok(Response::builder(404).body(RequestStatus { status: "CHANNEL_NOT_FOUND".into() }).build()),
                    Err(e) => Ok(Response::builder(422).body(Error422::from(e)).build())
                }
            },
            Err(_) => Ok(Response::builder(400).body(RequestStatus { status: "INVALID_CHANNELID".into() }).build())
        }
    }

}

mod tickets {
    use core_crypto::types::Hash;
    use core_types::channels::Ticket;
    use utils_types::traits::ToHex;
    use super::*;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ChannelTicket {
        channel_id: Hash,
        amount: Balance,
        index: u64,
        index_offset: u32,
        win_prob: String,
        channel_epoch: u32,
        signature: String,
    }

    impl From<Ticket> for ChannelTicket {
        fn from(value: Ticket) -> Self {
            Self {
                channel_id: value.channel_id,
                amount: value.amount,
                index: value.index,
                index_offset: value.index_offset,
                win_prob: value.win_prob().to_string(),
                channel_epoch: value.channel_epoch,
                signature: value.signature.expect("impossible to have an unsigned ticket").to_hex()
            }
        }
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/channels/{{channelId}}/tickets"),
        responses(
        (status = 200, description = "Channel funded successfully", body = Vec<ChannelTicket>),
        (status = 400, description = "Invalid channel id.", body = RequestStatus),
        (status = 404, description = "Channel not found.", body = RequestStatus),
        Error422,
        ),
        tag = "Channels"
    )]
    pub(super) async fn show_channel_tickets(req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();

        match Hash::from_hex(req.param("channelId")?) {
            Ok(channel_id) => match hopr.tickets_in_channel(&channel_id).await {
                Ok(tickets) => {
                    Ok(Response::builder(200)
                        .body(json!(tickets.into_iter().map(|t| ChannelTicket::from(t.ticket)).collect::<Vec<_>>()))
                        .build())
                },
                // TODO: impossible to distinguish when the channel does not exists
                Err(e) => Ok(Response::builder(422).body(Error422::from(e)).build())
            },
            Err(_) => Ok(Response::builder(400).body(RequestStatus { status: "INVALID_CHANNELID".into() }).build())
        }
    }

    #[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/tickets"),
        responses(
            (status = 200, description = "Channel funded successfully", body = Vec<ChannelTicket>),
            Error422,
        ),
        tag = "Tickets"
    )]
    pub(super) async fn show_all_tickets(req: Request<State<'_>>) -> tide::Result<Response> {
        let hopr = req.state().hopr.clone();
        match hopr.all_tickets().await {
            Ok(tickets) => {
                Ok(Response::builder(200)
                    .body(json!(tickets.into_iter().map(ChannelTicket::from).collect::<Vec<_>>()))
                    .build())
            },
            Err(e) => Ok(Response::builder(422).body(Error422::from(e)).build())
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

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AccountGetAddressResponse {
//     /// Addresses fetched successfully.
//     AddressesFetchedSuccessfully(models::AccountGetAddresses200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AccountGetAddressesResponse {
//     /// Addresses fetched successfully.
//     AddressesFetchedSuccessfully(models::AccountGetAddresses200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AccountGetBalancesResponse {
//     /// Balances fetched successfuly.
//     BalancesFetchedSuccessfuly(models::AccountGetBalances200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AccountWithdrawResponse {
//     /// Withdraw successful. Receipt from this response can be used to check details of the transaction on ethereum chain.
//     WithdrawSuccessful(models::AccountWithdraw200Response),
//     /// Incorrect data in request body. Make sure to provide valid currency ('NATIVE' | 'HOPR') or amount.
//     IncorrectDataInRequestBody(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Withdraw amount exeeds current balance or unknown error. You can check current balance using /account/balance endpoint.
//     WithdrawAmountExeedsCurrentBalanceOrUnknownError(models::AccountWithdraw422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AliasesGetAliasResponse {
//     /// HOPR address was found for the provided alias.
//     HOPRAddressWasFoundForTheProvidedAlias(models::AliasesGetAlias200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// This alias was not assigned to any PeerId before. You can get the list of all PeerId's and thier corresponding aliases using /aliases endpoint.
//     ThisAliasWasNotAssignedToAnyPeerIdBefore(models::RequestStatus),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AliasesGetAliasesResponse {
//     /// Returns List of Aliases and corresponding peerIds.
//     ReturnsListOfAliasesAndCorrespondingPeerIds(models::AliasesGetAliases200Response),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AliasesRemoveAliasResponse {
//     /// Alias removed succesfully.
//     AliasRemovedSuccesfully,
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum AliasesSetAliasResponse {
//     /// Alias set succesfully
//     AliasSetSuccesfully,
//     /// Invalid peerId. The format or length of the peerId is incorrect.
//     InvalidPeerId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsAggregateTicketsResponse {
//     /// Tickets successfully aggregated
//     TicketsSuccessfullyAggregated,
//     /// Invalid channel id.
//     InvalidChannelId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// The specified resource was not found
//     TheSpecifiedResourceWasNotFound,
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsCloseChannelResponse {
//     /// Channel closed succesfully.
//     ChannelClosedSuccesfully(models::ChannelsCloseChannel200Response),
//     /// Invalid channel id.
//     InvalidChannelId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsFundChannelResponse {
//     /// Channel funded successfully.
//     ChannelFundedSuccessfully(models::ChannelsFundChannel200Response),
//     /// Invalid channel id.
//     InvalidChannelId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// The specified resource was not found
//     TheSpecifiedResourceWasNotFound,
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsGetChannelResponse {
//     /// Channel fetched succesfully.
//     ChannelFetchedSuccesfully(models::ChannelTopology),
//     /// Invalid channel id.
//     InvalidChannelId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// The specified resource was not found
//     TheSpecifiedResourceWasNotFound,
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsGetChannelsResponse {
//     /// Channels fetched successfully.
//     ChannelsFetchedSuccessfully(models::ChannelsGetChannels200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsGetTicketsResponse {
//     /// Tickets fetched successfully.
//     TicketsFetchedSuccessfully(Vec<models::Ticket>),
//     /// Invalid peerId.
//     InvalidPeerId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Tickets were not found for that channel. That means that no messages were sent inside this channel yet.
//     TicketsWereNotFoundForThatChannel(models::RequestStatus),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsOpenChannelResponse {
//     /// Channel succesfully opened.
//     ChannelSuccesfullyOpened(models::ChannelsOpenChannel201Response),
//     /// Problem with inputs.
//     ProblemWithInputs(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// Failed to open the channel because of insufficient HOPR balance or allowance.
//     FailedToOpenTheChannelBecauseOfInsufficientHOPRBalanceOrAllowance(models::ChannelsOpenChannel403Response),
//     /// Failed to open the channel because the channel between this nodes already exists.
//     FailedToOpenTheChannelBecauseTheChannelBetweenThisNodesAlreadyExists(models::ChannelsOpenChannel409Response),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum ChannelsRedeemTicketsResponse {
//     /// Tickets redeemed successfully.
//     TicketsRedeemedSuccessfully,
//     /// Invalid channel id.
//     InvalidChannelId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Tickets were not found for that channel. That means that no messages were sent inside this channel yet.
//     TicketsWereNotFoundForThatChannel(models::RequestStatus),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum CheckNodeHealthyResponse {
//     /// The node is ready
//     TheNodeIsReady(serde_json::Value),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum CheckNodeReadyResponse {
//     /// The node is ready
//     TheNodeIsReady(serde_json::Value),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum CheckNodeStartedResponse {
//     /// The node is started
//     TheNodeIsStarted(serde_json::Value),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum MessagesDeleteMessagesResponse {
//     /// Messages successfully deleted.
//     MessagesSuccessfullyDeleted,
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum MessagesGetSizeResponse {
//     /// Returns the message inbox size filtered by the given tag.
//     ReturnsTheMessageInboxSizeFilteredByTheGivenTag(models::MessagesGetSize200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum MessagesPopAllMessageResponse {
//     /// Returns list of messages.
//     ReturnsListOfMessages(models::MessagesPopAllMessage200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum MessagesPopMessageResponse {
//     /// Returns a message.
//     ReturnsAMessage(models::ReceivedMessage),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// The specified resource was not found
//     TheSpecifiedResourceWasNotFound,
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum MessagesSendMessageResponse {
//     /// The message was sent successfully. NOTE: This does not imply successful delivery.
//     TheMessageWasSentSuccessfully(String),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum MessagesWebsocketResponse {
//     /// Switching protocols
//     SwitchingProtocols,
//     /// Incoming data
//     IncomingData(String),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Not found
//     NotFound,
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum NodeGetEntryNodesResponse {
//     /// Entry node information fetched successfuly.
//     EntryNodeInformationFetchedSuccessfuly(
//         std::collections::HashMap<String, models::NodeGetEntryNodes200ResponseValue>,
//     ),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum NodeGetInfoResponse {
//     /// Node information fetched successfuly.
//     NodeInformationFetchedSuccessfuly(models::NodeGetInfo200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum NodeGetMetricsResponse {
//     /// Returns the encoded serialized metrics.
//     ReturnsTheEncodedSerializedMetrics(String),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum NodeGetPeersResponse {
//     /// Peers information fetched successfuly.
//     PeersInformationFetchedSuccessfuly(models::NodeGetPeers200Response),
//     /// Invalid input. One of the parameters passed is in an incorrect format.
//     InvalidInput(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum NodeGetVersionResponse {
//     /// Returns the release version of the running node.
//     ReturnsTheReleaseVersionOfTheRunningNode(String),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum PeerInfoGetPeerInfoResponse {
//     /// Peer information fetched successfully.
//     PeerInformationFetchedSuccessfully(models::PeerInfoGetPeerInfo200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum PeersPingPeerResponse {
//     /// Ping successful.
//     PingSuccessful(models::PeersPingPeer200Response),
//     /// Invalid peerId.
//     InvalidPeerId(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// An error occured (see error details) or timeout - node with specified PeerId didn't respond in time.
//     AnErrorOccured(models::RequestStatus),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum SettingsGetSettingsResponse {
//     /// Settings fetched succesfully.
//     SettingsFetchedSuccesfully(models::Settings),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum SettingsSetSettingResponse {
//     /// Setting set succesfully
//     SettingSetSuccesfully,
//     /// Invalid input. Either setting with that name doesn't exist or the value is incorrect.
//     InvalidInput(models::RequestStatus),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum TicketsGetStatisticsResponse {
//     /// Tickets statistics fetched successfully. Check schema for description of every field in the statistics.
//     TicketsStatisticsFetchedSuccessfully(models::TicketsGetStatistics200Response),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum TicketsGetTicketsResponse {
//     /// Tickets fetched successfully.
//     TicketsFetchedSuccessfully(Vec<models::Ticket>),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum TicketsRedeemTicketsResponse {
//     /// Tickets redeemed succesfully.
//     TicketsRedeemedSuccesfully,
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum TokensCreateResponse {
//     /// Token succesfully created.
//     TokenSuccesfullyCreated(models::TokensCreate201Response),
//     /// Problem with inputs.
//     ProblemWithInputs(models::RequestStatus),
//     /// Missing capability to access endpoint
//     MissingCapabilityToAccessEndpoint,
//     /// Unknown failure.
//     UnknownFailure(models::TokensCreate422Response),
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum TokensDeleteResponse {
//     /// Token successfully deleted.
//     TokenSuccessfullyDeleted,
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// The specified resource was not found
//     TheSpecifiedResourceWasNotFound,
// }

// #[derive(Debug, PartialEq, Serialize, Deserialize)]
// #[must_use]
// pub enum TokensGetTokenResponse {
//     /// Token information.
//     TokenInformation(models::Token),
//     /// authentication failed
//     AuthenticationFailed(models::Error),
//     /// authorization failed
//     AuthorizationFailed(models::Error),
//     /// The specified resource was not found
//     TheSpecifiedResourceWasNotFound,
// }

// /// API
// #[async_trait]
// #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
// pub trait Api<C: Send + Sync> {
//     fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
//         Poll::Ready(Ok(()))
//     }

//     async fn account_get_address(&self, context: &C) -> Result<AccountGetAddressResponse, ApiError>;

//     async fn account_get_addresses(&self, context: &C) -> Result<AccountGetAddressesResponse, ApiError>;

//     async fn account_get_balances(&self, context: &C) -> Result<AccountGetBalancesResponse, ApiError>;

//     async fn account_withdraw(
//         &self,
//         account_withdraw_request: Option<models::AccountWithdrawRequest>,
//         context: &C,
//     ) -> Result<AccountWithdrawResponse, ApiError>;

//     async fn aliases_get_alias(&self, alias: String, context: &C) -> Result<AliasesGetAliasResponse, ApiError>;

//     async fn aliases_get_aliases(&self, context: &C) -> Result<AliasesGetAliasesResponse, ApiError>;

//     async fn aliases_remove_alias(&self, alias: String, context: &C) -> Result<AliasesRemoveAliasResponse, ApiError>;

//     async fn aliases_set_alias(
//         &self,
//         aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
//         context: &C,
//     ) -> Result<AliasesSetAliasResponse, ApiError>;

//     async fn channels_aggregate_tickets(
//         &self,
//         channelid: String,
//         context: &C,
//     ) -> Result<ChannelsAggregateTicketsResponse, ApiError>;

//     async fn channels_close_channel(
//         &self,
//         channelid: String,
//         context: &C,
//     ) -> Result<ChannelsCloseChannelResponse, ApiError>;

//     async fn channels_fund_channel(
//         &self,
//         channelid: String,
//         channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
//         context: &C,
//     ) -> Result<ChannelsFundChannelResponse, ApiError>;

//     async fn channels_get_channel(
//         &self,
//         channelid: serde_json::Value,
//         context: &C,
//     ) -> Result<ChannelsGetChannelResponse, ApiError>;

//     async fn channels_get_channels(
//         &self,
//         including_closed: Option<String>,
//         full_topology: Option<String>,
//         context: &C,
//     ) -> Result<ChannelsGetChannelsResponse, ApiError>;

//     async fn channels_get_tickets(
//         &self,
//         channelid: String,
//         context: &C,
//     ) -> Result<ChannelsGetTicketsResponse, ApiError>;

//     async fn channels_open_channel(
//         &self,
//         channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
//         context: &C,
//     ) -> Result<ChannelsOpenChannelResponse, ApiError>;

//     async fn channels_redeem_tickets(
//         &self,
//         channelid: String,
//         context: &C,
//     ) -> Result<ChannelsRedeemTicketsResponse, ApiError>;

//     async fn check_node_healthy(&self, context: &C) -> Result<CheckNodeHealthyResponse, ApiError>;

//     async fn check_node_ready(&self, context: &C) -> Result<CheckNodeReadyResponse, ApiError>;

//     async fn check_node_started(&self, context: &C) -> Result<CheckNodeStartedResponse, ApiError>;

//     async fn messages_delete_messages(&self, tag: i32, context: &C)
//         -> Result<MessagesDeleteMessagesResponse, ApiError>;

//     async fn messages_get_size(&self, tag: i32, context: &C) -> Result<MessagesGetSizeResponse, ApiError>;

//     async fn messages_pop_all_message(
//         &self,
//         messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
//         context: &C,
//     ) -> Result<MessagesPopAllMessageResponse, ApiError>;

//     async fn messages_pop_message(
//         &self,
//         messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
//         context: &C,
//     ) -> Result<MessagesPopMessageResponse, ApiError>;

//     async fn messages_send_message(
//         &self,
//         messages_send_message_request: Option<models::MessagesSendMessageRequest>,
//         context: &C,
//     ) -> Result<MessagesSendMessageResponse, ApiError>;

//     async fn messages_websocket(&self, context: &C) -> Result<MessagesWebsocketResponse, ApiError>;

//     async fn node_get_entry_nodes(&self, context: &C) -> Result<NodeGetEntryNodesResponse, ApiError>;

//     async fn node_get_info(&self, context: &C) -> Result<NodeGetInfoResponse, ApiError>;

//     async fn node_get_metrics(&self, context: &C) -> Result<NodeGetMetricsResponse, ApiError>;

//     async fn node_get_peers(&self, quality: Option<f64>, context: &C) -> Result<NodeGetPeersResponse, ApiError>;

//     async fn node_get_version(&self, context: &C) -> Result<NodeGetVersionResponse, ApiError>;

//     async fn peer_info_get_peer_info(
//         &self,
//         peerid: String,
//         context: &C,
//     ) -> Result<PeerInfoGetPeerInfoResponse, ApiError>;

//     async fn peers_ping_peer(&self, peerid: String, context: &C) -> Result<PeersPingPeerResponse, ApiError>;

//     async fn settings_get_settings(&self, context: &C) -> Result<SettingsGetSettingsResponse, ApiError>;

//     async fn settings_set_setting(
//         &self,
//         setting: String,
//         settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
//         context: &C,
//     ) -> Result<SettingsSetSettingResponse, ApiError>;

//     async fn tickets_get_statistics(&self, context: &C) -> Result<TicketsGetStatisticsResponse, ApiError>;

//     async fn tickets_get_tickets(&self, context: &C) -> Result<TicketsGetTicketsResponse, ApiError>;

//     async fn tickets_redeem_tickets(&self, context: &C) -> Result<TicketsRedeemTicketsResponse, ApiError>;

//     async fn tokens_create(
//         &self,
//         tokens_create_request: Option<models::TokensCreateRequest>,
//         context: &C,
//     ) -> Result<TokensCreateResponse, ApiError>;

//     async fn tokens_delete(&self, id: String, context: &C) -> Result<TokensDeleteResponse, ApiError>;

//     async fn tokens_get_token(&self, context: &C) -> Result<TokensGetTokenResponse, ApiError>;
// }

// /// API where `Context` isn't passed on every API call
// #[async_trait]
// #[allow(clippy::too_many_arguments, clippy::ptr_arg)]
// pub trait ApiNoContext<C: Send + Sync> {
//     fn poll_ready(&self, _cx: &mut Context) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

//     fn context(&self) -> &C;

//     async fn account_get_address(&self) -> Result<AccountGetAddressResponse, ApiError>;

//     async fn account_get_addresses(&self) -> Result<AccountGetAddressesResponse, ApiError>;

//     async fn account_get_balances(&self) -> Result<AccountGetBalancesResponse, ApiError>;

//     async fn account_withdraw(
//         &self,
//         account_withdraw_request: Option<models::AccountWithdrawRequest>,
//     ) -> Result<AccountWithdrawResponse, ApiError>;

//     async fn aliases_get_alias(&self, alias: String) -> Result<AliasesGetAliasResponse, ApiError>;

//     async fn aliases_get_aliases(&self) -> Result<AliasesGetAliasesResponse, ApiError>;

//     async fn aliases_remove_alias(&self, alias: String) -> Result<AliasesRemoveAliasResponse, ApiError>;

//     async fn aliases_set_alias(
//         &self,
//         aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
//     ) -> Result<AliasesSetAliasResponse, ApiError>;

//     async fn channels_aggregate_tickets(&self, channelid: String)
//         -> Result<ChannelsAggregateTicketsResponse, ApiError>;

//     async fn channels_close_channel(&self, channelid: String) -> Result<ChannelsCloseChannelResponse, ApiError>;

//     async fn channels_fund_channel(
//         &self,
//         channelid: String,
//         channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
//     ) -> Result<ChannelsFundChannelResponse, ApiError>;

//     async fn channels_get_channel(&self, channelid: serde_json::Value) -> Result<ChannelsGetChannelResponse, ApiError>;

//     async fn channels_get_channels(
//         &self,
//         including_closed: Option<String>,
//         full_topology: Option<String>,
//     ) -> Result<ChannelsGetChannelsResponse, ApiError>;

//     async fn channels_get_tickets(&self, channelid: String) -> Result<ChannelsGetTicketsResponse, ApiError>;

//     async fn channels_open_channel(
//         &self,
//         channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
//     ) -> Result<ChannelsOpenChannelResponse, ApiError>;

//     async fn channels_redeem_tickets(&self, channelid: String) -> Result<ChannelsRedeemTicketsResponse, ApiError>;

//     async fn check_node_healthy(&self) -> Result<CheckNodeHealthyResponse, ApiError>;

//     async fn check_node_ready(&self) -> Result<CheckNodeReadyResponse, ApiError>;

//     async fn check_node_started(&self) -> Result<CheckNodeStartedResponse, ApiError>;

//     async fn messages_delete_messages(&self, tag: i32) -> Result<MessagesDeleteMessagesResponse, ApiError>;

//     async fn messages_get_size(&self, tag: i32) -> Result<MessagesGetSizeResponse, ApiError>;

//     async fn messages_pop_all_message(
//         &self,
//         messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
//     ) -> Result<MessagesPopAllMessageResponse, ApiError>;

//     async fn messages_pop_message(
//         &self,
//         messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
//     ) -> Result<MessagesPopMessageResponse, ApiError>;

//     async fn messages_send_message(
//         &self,
//         messages_send_message_request: Option<models::MessagesSendMessageRequest>,
//     ) -> Result<MessagesSendMessageResponse, ApiError>;

//     async fn messages_websocket(&self) -> Result<MessagesWebsocketResponse, ApiError>;

//     async fn node_get_entry_nodes(&self) -> Result<NodeGetEntryNodesResponse, ApiError>;

//     async fn node_get_info(&self) -> Result<NodeGetInfoResponse, ApiError>;

//     async fn node_get_metrics(&self) -> Result<NodeGetMetricsResponse, ApiError>;

//     async fn node_get_peers(&self, quality: Option<f64>) -> Result<NodeGetPeersResponse, ApiError>;

//     async fn node_get_version(&self) -> Result<NodeGetVersionResponse, ApiError>;

//     async fn peer_info_get_peer_info(&self, peerid: String) -> Result<PeerInfoGetPeerInfoResponse, ApiError>;

//     async fn peers_ping_peer(&self, peerid: String) -> Result<PeersPingPeerResponse, ApiError>;

//     async fn settings_get_settings(&self) -> Result<SettingsGetSettingsResponse, ApiError>;

//     async fn settings_set_setting(
//         &self,
//         setting: String,
//         settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
//     ) -> Result<SettingsSetSettingResponse, ApiError>;

//     async fn tickets_get_statistics(&self) -> Result<TicketsGetStatisticsResponse, ApiError>;

//     async fn tickets_get_tickets(&self) -> Result<TicketsGetTicketsResponse, ApiError>;

//     async fn tickets_redeem_tickets(&self) -> Result<TicketsRedeemTicketsResponse, ApiError>;

//     async fn tokens_create(
//         &self,
//         tokens_create_request: Option<models::TokensCreateRequest>,
//     ) -> Result<TokensCreateResponse, ApiError>;

//     async fn tokens_delete(&self, id: String) -> Result<TokensDeleteResponse, ApiError>;

//     async fn tokens_get_token(&self) -> Result<TokensGetTokenResponse, ApiError>;
// }

// /// Trait to extend an API to make it easy to bind it to a context.
// pub trait ContextWrapperExt<C: Send + Sync>
// where
//     Self: Sized,
// {
//     /// Binds this API to a context.
//     fn with_context(self, context: C) -> ContextWrapper<Self, C>;
// }

// impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
//     fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
//         ContextWrapper::<T, C>::new(self, context)
//     }
// }

// #[async_trait]
// impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
//     fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
//         self.api().poll_ready(cx)
//     }

//     fn context(&self) -> &C {
//         ContextWrapper::context(self)
//     }

//     async fn account_get_address(&self) -> Result<AccountGetAddressResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().account_get_address(&context).await
//     }

//     async fn account_get_addresses(&self) -> Result<AccountGetAddressesResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().account_get_addresses(&context).await
//     }

//     async fn account_get_balances(&self) -> Result<AccountGetBalancesResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().account_get_balances(&context).await
//     }

//     async fn account_withdraw(
//         &self,
//         account_withdraw_request: Option<models::AccountWithdrawRequest>,
//     ) -> Result<AccountWithdrawResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().account_withdraw(account_withdraw_request, &context).await
//     }

//     async fn aliases_get_alias(&self, alias: String) -> Result<AliasesGetAliasResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().aliases_get_alias(alias, &context).await
//     }

//     async fn aliases_get_aliases(&self) -> Result<AliasesGetAliasesResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().aliases_get_aliases(&context).await
//     }

//     async fn aliases_remove_alias(&self, alias: String) -> Result<AliasesRemoveAliasResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().aliases_remove_alias(alias, &context).await
//     }

//     async fn aliases_set_alias(
//         &self,
//         aliases_set_alias_request: Option<models::AliasesSetAliasRequest>,
//     ) -> Result<AliasesSetAliasResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().aliases_set_alias(aliases_set_alias_request, &context).await
//     }

//     async fn channels_aggregate_tickets(
//         &self,
//         channelid: String,
//     ) -> Result<ChannelsAggregateTicketsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().channels_aggregate_tickets(channelid, &context).await
//     }

//     async fn channels_close_channel(&self, channelid: String) -> Result<ChannelsCloseChannelResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().channels_close_channel(channelid, &context).await
//     }

//     async fn channels_fund_channel(
//         &self,
//         channelid: String,
//         channels_fund_channel_request: Option<models::ChannelsFundChannelRequest>,
//     ) -> Result<ChannelsFundChannelResponse, ApiError> {
//         let context = self.context().clone();
//         self.api()
//             .channels_fund_channel(channelid, channels_fund_channel_request, &context)
//             .await
//     }

//     async fn channels_get_channel(&self, channelid: serde_json::Value) -> Result<ChannelsGetChannelResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().channels_get_channel(channelid, &context).await
//     }

//     async fn channels_get_channels(
//         &self,
//         including_closed: Option<String>,
//         full_topology: Option<String>,
//     ) -> Result<ChannelsGetChannelsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api()
//             .channels_get_channels(including_closed, full_topology, &context)
//             .await
//     }

//     async fn channels_get_tickets(&self, channelid: String) -> Result<ChannelsGetTicketsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().channels_get_tickets(channelid, &context).await
//     }

//     async fn channels_open_channel(
//         &self,
//         channels_open_channel_request: Option<models::ChannelsOpenChannelRequest>,
//     ) -> Result<ChannelsOpenChannelResponse, ApiError> {
//         let context = self.context().clone();
//         self.api()
//             .channels_open_channel(channels_open_channel_request, &context)
//             .await
//     }

//     async fn channels_redeem_tickets(&self, channelid: String) -> Result<ChannelsRedeemTicketsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().channels_redeem_tickets(channelid, &context).await
//     }

//     async fn check_node_healthy(&self) -> Result<CheckNodeHealthyResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().check_node_healthy(&context).await
//     }

//     async fn check_node_ready(&self) -> Result<CheckNodeReadyResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().check_node_ready(&context).await
//     }

//     async fn check_node_started(&self) -> Result<CheckNodeStartedResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().check_node_started(&context).await
//     }

//     async fn messages_delete_messages(&self, tag: i32) -> Result<MessagesDeleteMessagesResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().messages_delete_messages(tag, &context).await
//     }

//     async fn messages_get_size(&self, tag: i32) -> Result<MessagesGetSizeResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().messages_get_size(tag, &context).await
//     }

//     async fn messages_pop_all_message(
//         &self,
//         messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
//     ) -> Result<MessagesPopAllMessageResponse, ApiError> {
//         let context = self.context().clone();
//         self.api()
//             .messages_pop_all_message(messages_pop_all_message_request, &context)
//             .await
//     }

//     async fn messages_pop_message(
//         &self,
//         messages_pop_all_message_request: Option<models::MessagesPopAllMessageRequest>,
//     ) -> Result<MessagesPopMessageResponse, ApiError> {
//         let context = self.context().clone();
//         self.api()
//             .messages_pop_message(messages_pop_all_message_request, &context)
//             .await
//     }

//     async fn messages_send_message(
//         &self,
//         messages_send_message_request: Option<models::MessagesSendMessageRequest>,
//     ) -> Result<MessagesSendMessageResponse, ApiError> {
//         let context = self.context().clone();
//         self.api()
//             .messages_send_message(messages_send_message_request, &context)
//             .await
//     }

//     async fn messages_websocket(&self) -> Result<MessagesWebsocketResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().messages_websocket(&context).await
//     }

//     async fn node_get_entry_nodes(&self) -> Result<NodeGetEntryNodesResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().node_get_entry_nodes(&context).await
//     }

//     async fn node_get_info(&self) -> Result<NodeGetInfoResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().node_get_info(&context).await
//     }

//     async fn node_get_metrics(&self) -> Result<NodeGetMetricsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().node_get_metrics(&context).await
//     }

//     async fn node_get_peers(&self, quality: Option<f64>) -> Result<NodeGetPeersResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().node_get_peers(quality, &context).await
//     }

//     async fn node_get_version(&self) -> Result<NodeGetVersionResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().node_get_version(&context).await
//     }

//     async fn peer_info_get_peer_info(&self, peerid: String) -> Result<PeerInfoGetPeerInfoResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().peer_info_get_peer_info(peerid, &context).await
//     }

//     async fn peers_ping_peer(&self, peerid: String) -> Result<PeersPingPeerResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().peers_ping_peer(peerid, &context).await
//     }

//     async fn settings_get_settings(&self) -> Result<SettingsGetSettingsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().settings_get_settings(&context).await
//     }

//     async fn settings_set_setting(
//         &self,
//         setting: String,
//         settings_set_setting_request: Option<models::SettingsSetSettingRequest>,
//     ) -> Result<SettingsSetSettingResponse, ApiError> {
//         let context = self.context().clone();
//         self.api()
//             .settings_set_setting(setting, settings_set_setting_request, &context)
//             .await
//     }

//     async fn tickets_get_statistics(&self) -> Result<TicketsGetStatisticsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().tickets_get_statistics(&context).await
//     }

//     async fn tickets_get_tickets(&self) -> Result<TicketsGetTicketsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().tickets_get_tickets(&context).await
//     }

//     async fn tickets_redeem_tickets(&self) -> Result<TicketsRedeemTicketsResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().tickets_redeem_tickets(&context).await
//     }

//     async fn tokens_create(
//         &self,
//         tokens_create_request: Option<models::TokensCreateRequest>,
//     ) -> Result<TokensCreateResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().tokens_create(tokens_create_request, &context).await
//     }

//     async fn tokens_delete(&self, id: String) -> Result<TokensDeleteResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().tokens_delete(id, &context).await
//     }

//     async fn tokens_get_token(&self) -> Result<TokensGetTokenResponse, ApiError> {
//         let context = self.context().clone();
//         self.api().tokens_get_token(&context).await
//     }
// }

// #[cfg(feature = "client")]
// pub mod client;

// // Re-export Client as a top-level name
// #[cfg(feature = "client")]
// pub use client::Client;

// #[cfg(feature = "server")]
// pub mod server;

// // Re-export router() as a top-level name
// #[cfg(feature = "server")]
// pub use self::server::Service;

// #[cfg(feature = "server")]
// pub mod context;

// pub mod models;

// #[cfg(any(feature = "client", feature = "server"))]
// pub(crate) mod header;
