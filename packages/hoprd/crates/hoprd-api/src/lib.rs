use std::{sync::Arc, collections::HashMap};

use async_std::sync::RwLock;
use libp2p_identity::PeerId;
use serde_json::json;
use serde_with::{serde_as, DisplayFromStr};
use tide::{http::Mime, Request, Response};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::Config;

use hopr_lib::{Address, Balance, BalanceType, Hopr};

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

        api.at("/messages/")
            .post(messages::send_message)
            .delete(messages::delete_messages);
        api.at("/messages/pop").get(messages::pop);
        api.at("/messages/pop-all").get(messages::pop_all);
        api.at("/messages/peek").get(messages::peek);
        api.at("/messages/peek-all").get(messages::peek_all);
        api.at("/messages/size").get(messages::size);

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RequestStatus {
    pub status: String,
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
    pub async fn aliases(req: Request<State<'_>>) -> tide::Result<Response> {
        let aliases = req.state().aliases.clone();

        let aliases = aliases.read()
            .await
            .iter()
            .map(|(key, value)| {
                AliasPeerId {
                    alias: key.clone(),
                    peer_id: value.clone(),
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

        aliases.write().await.insert(args.alias, args.peer_id);
        Ok(Response::builder(200)
            .body(json!(PeerIdArg{peer_id: args.peer_id}))
            .build()
        )
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
                .body(json!(PeerIdArg{peer_id: peer_id.clone()}))
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
            Err(e) => return Ok(Response::builder(422).body(json!(Error422::new(e.to_string()))).build()),
        }

        match hopr.get_balance(BalanceType::HOPR).await {
            Ok(v) => account_balances.hopr = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(json!(Error422::new(e.to_string()))).build()),
        }

        match hopr.get_safe_balance(BalanceType::Native).await {
            Ok(v) => account_balances.safe_native = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(json!(Error422::new(e.to_string()))).build()),
        }

        match hopr.get_safe_balance(BalanceType::HOPR).await {
            Ok(v) => account_balances.safe_hopr = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(json!(Error422::new(e.to_string()))).build()),
        }

        match hopr.safe_allowance().await {
            Ok(v) => account_balances.safe_hopr_allowance = v.to_string(),
            Err(e) => return Ok(Response::builder(422).body(json!(Error422::new(e.to_string()))).build()),
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
            Err(e) => Ok(Response::builder(422).body(json!(Error422::new(e.to_string()))).build()),
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
        #[validate(range(min=1, max=3))]
        pub hops: Option<u16>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]

    struct SendMessageRes {
        pub challenge: HalfKeyChallenge,
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
            Error422
        ),
        tag = "Messages"
    )]
    pub async fn send_message(mut req: Request<State<'_>>) -> tide::Result<Response> {
        let args: SendMessageReq = req.body_json().await?;
        let hopr = req.state().hopr.clone();

        if let Some(path) = &args.path {
            if path.len() > 3 {
                return Ok(Response::builder(422).body(json!(Error422::new("The path components must contain at most 3 elements".into()))).build())
            }
        }

        match hopr.send_message(Box::from(args.body.as_ref()), args.peer_id, args.path, args.hops, Some(args.tag)).await {
            Ok(challenge) => Ok(Response::builder(202).body(json!(SendMessageRes{challenge})).build()),
            Err(e) => Ok(Response::builder(422).body(json!(Error422::new(e.to_string()))).build()),
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
    pub async fn delete_messages(req: Request<State<'_>>) -> tide::Result<Response> {
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
    pub async fn size(req: Request<State<'_>>) -> tide::Result<Response> {
        let tag: Tag = req.query()?;
        let inbox = req.state().inbox.clone();

        let size = inbox.read().await.size(Some(tag.tag)).await;

        Ok(Response::builder(200).body(json!(Size{size})).build())
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct MessagePopRes {
        tag: u16,
        body: String,
        received_at: u128
    }

    fn to_api_message(data: hopr_lib::ApplicationData, ts: Duration) -> Result<MessagePopRes, String> {
        if data.application_tag.is_none() {
            Err("No application tag was present despite picking from a tagged inbox".into())
        } else {
            match std::str::from_utf8(&data.plain_text) {
                Ok(data_str) => {
                    Ok(MessagePopRes{
                        tag: data.application_tag.unwrap_or(0),
                        body: data_str.into(),
                        received_at: ts.as_millis()
                    })
                },
                Err(error) => {
                    Err(format!("Failed to deserialize data into string: {error}"))
                }
            }
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
            Error422
        ),
        tag = "Messages"
    )]
    pub async fn pop(mut req: Request<State<'_>>) -> tide::Result<Response> {
        let tag: Tag = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        if let Some((data, ts)) = inbox.pop(Some(tag.tag)).await {
            match to_api_message(data, ts) {
                Ok(message) => Ok(Response::builder(204).body(json!(message)).build()),
                Err(e) => Ok(Response::builder(422).body(json!(Error422::new(e))).build())
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
            Error422
        ),
        tag = "Messages"
    )]
    pub async fn pop_all(mut req: Request<State<'_>>) -> tide::Result<Response> {
        let tag: Tag = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        let messages = inbox.pop_all(Some(tag.tag))
            .await
            .into_iter()
            .filter_map(|(data, ts)| {
                to_api_message(data, ts).ok()
            })
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
            Error422
        ),
        tag = "Messages"
    )]
    pub async fn peek(mut req: Request<State<'_>>) -> tide::Result<Response> {
        let tag: Tag = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        if let Some((data, ts)) = inbox.pop(Some(tag.tag)).await {
            match to_api_message(data, ts) {
                Ok(message) => Ok(Response::builder(204).body(json!(message)).build()),
                Err(e) => Ok(Response::builder(422).body(json!(Error422::new(e))).build())
            }
        } else {
            Ok(Response::builder(404).build())
        }
    }

    /// Peek the list of messages currently present in the nodes message inbox.
    /// 
    /// The messages are not removed from the inbox.
    #[utoipa::path(
        post,
        path = const_format::formatcp!("{}/messages/peek-all", BASE_PATH),
        responses(
            (status = 200, description = "All messages successfully peeked at.", body = [MessagePopRes]),
            (status = 404, description = "The specified resource was not found."),
            Error422
        ),
        tag = "Messages"
    )]
    pub async fn peek_all(mut req: Request<State<'_>>) -> tide::Result<Response> {
        let tag: Tag = req.body_json().await?;
        let inbox = req.state().inbox.clone();

        let inbox = inbox.write().await;
        let messages = inbox.peek_all(Some(tag.tag))
            .await
            .into_iter()
            .filter_map(|(data, ts)| {
                to_api_message(data, ts).ok()
            })
            .collect::<Vec<_>>();

        Ok(Response::builder(200).body(json!(messages)).build())
    }
}

mod tickets {}
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