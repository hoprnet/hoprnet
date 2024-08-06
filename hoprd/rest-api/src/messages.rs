use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, Query, State,
    },
    http::status::StatusCode,
    response::IntoResponse,
    Error,
};
use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};
use futures_concurrency::stream::Merge;
use libp2p_identity::PeerId;
use serde::Deserialize;
use serde_json::json;
use serde_with::{serde_as, Bytes, DisplayFromStr, DurationMilliSeconds};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error, warn};
use validator::Validate;

use hopr_lib::{AsUnixTimestamp, HalfKeyChallenge, PathOptions, TransportOutput, RESERVED_TAG_UPPER_LIMIT};

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

#[derive(Debug, Default, Clone, Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
pub(crate) struct TagQueryRequest {
    #[schema(required = false)]
    tag: Option<u16>,
}

#[derive(Debug, Clone, serde::Serialize, Deserialize, utoipa::ToSchema)]
pub(crate) struct SizeResponse {
    size: usize,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Deserialize, validator::Validate, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({
        "body": "Test message",
        "hops": 1,
        "path": [
            "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33"
        ],
        "peerId": "12D3KooWEDc1vGJevww48trVDDf6pr1f6N3F86sGJfQrKCyc8kJ1",
        "tag": 2000
    }))]
pub(crate) struct SendMessageBodyRequest {
    /// The message tag used to filter messages based on application
    tag: u16,
    /// Message to be transmitted over the network
    #[serde_as(as = "Bytes")]
    body: Vec<u8>,
    /// The recipient HOPR PeerId
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    peer_id: PeerId,
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    #[validate(length(min = 0, max = 3))]
    #[schema(value_type = Option<Vec<String>>)]
    path: Option<Vec<PeerId>>,
    #[validate(range(min = 0, max = 3))]
    hops: Option<u16>,
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
    challenge: HalfKeyChallenge,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[schema(value_type = u64)]
    timestamp: std::time::Duration,
}

#[serde_as]
#[derive(Debug, Default, Clone, Deserialize, utoipa::ToSchema)]
pub(crate) struct GetMessageBodyRequest {
    /// The message tag used to filter messages based on application
    #[schema(required = false)]
    tag: Option<u16>,
    /// Timestamp to filter messages received after this timestamp
    #[serde_as(as = "Option<DurationMilliSeconds<u64>>")]
    #[schema(required = false, value_type = u64)]
    timestamp: Option<std::time::Duration>,
}

/// Send a message to another peer using the given path.
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
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Messages",
    )]
pub(super) async fn send_message(
    State(state): State<Arc<InternalState>>,
    Json(args): Json<SendMessageBodyRequest>,
) -> impl IntoResponse {
    let hopr = state.hopr.clone();

    if let Err(e) = args.validate() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
            .into_response();
    }

    // Use the message encoder, if any
    let msg_body = state
        .msg_encoder
        .as_ref()
        .map(|enc| enc(&args.body))
        .unwrap_or_else(|| args.body.into_boxed_slice());

    let options = if let Some(intermediate_path) = args.path {
        PathOptions::IntermediatePath(intermediate_path)
    } else if let Some(hops) = args.hops {
        PathOptions::Hops(hops)
    } else {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure("One of either hops or intermediate path must be specified".to_string()),
        )
            .into_response();
    };

    let timestamp = std::time::SystemTime::now().as_unix_timestamp();

    match hopr.send_message(msg_body, args.peer_id, options, Some(args.tag)).await {
        Ok(challenge) => (StatusCode::ACCEPTED, Json(SendMessageResponse { challenge, timestamp })).into_response(),
        Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response(),
    }
}

#[derive(Debug, Default, Clone, Deserialize, utoipa::ToSchema)]
#[schema(value_type = String)] //, format = Binary)]
#[allow(dead_code)] // not dead code, just for codegen
struct Text(String);

#[derive(Debug, Clone, Deserialize)]
struct WebSocketSendMsg {
    cmd: String,
    args: SendMessageBodyRequest,
}

#[derive(Debug, Clone, serde::Serialize)]
struct WebSocketReadMsg {
    #[serde(rename = "type")]
    type_: String,
    tag: u16,
    body: String,
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
struct WebSocketReadAck {
    #[serde(rename = "type")]
    type_: String,
    #[serde_as(as = "DisplayFromStr")]
    id: HalfKeyChallenge,
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
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Messages",
    )]
pub(crate) async fn websocket(ws: WebSocketUpgrade, State(state): State<Arc<InternalState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket_connection(socket, state))
}

enum WebSocketInput {
    Network(TransportOutput),
    WsInput(core::result::Result<Message, Error>),
}

async fn websocket_connection(socket: WebSocket, state: Arc<InternalState>) {
    let (mut sender, receiver) = socket.split();

    let ws_rx = state.websocket_rx.activate_cloned();

    let mut queue = (
        receiver.map(WebSocketInput::WsInput),
        ws_rx.map(WebSocketInput::Network),
    )
        .merge();

    while let Some(v) = queue.next().await {
        match v {
            WebSocketInput::Network(net_in) => match net_in {
                TransportOutput::Received(data) => {
                    debug!("websocket notifying client: received msg");
                    match sender
                        .send(Message::Text(json!(WebSocketReadMsg::from(data)).to_string()))
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => error!("failed to send websocket message: {e}"),
                    };
                }
                TransportOutput::Sent(hkc) => {
                    debug!("websocket notifying client: received next hop receive confirmation");
                    match sender
                        .send(Message::Text(json!(WebSocketReadAck::from_ack(hkc)).to_string()))
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => error!("failed to send websocket message: {e}"),
                    };
                }
            },
            WebSocketInput::WsInput(ws_in) => match ws_in {
                Ok(Message::Text(input)) => match handle_send_message(&input, state.clone(), &mut sender).await {
                    Ok(_) => {}
                    Err(e) => error!("failed to send message: {e}"),
                },
                Ok(Message::Close(_)) => {
                    debug!("received close frame, closing connection");
                    break;
                }
                Err(e) => {
                    error!("failed to get a valid websocket message: {e}, closing connection");
                    break;
                }
                Ok(m) => warn!("skipping an unsupported websocket message: {:?}", m),
            },
        }
    }
}

async fn handle_send_message(
    input: &str,
    state: Arc<InternalState>,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<(), String> {
    match serde_json::from_str(input) {
        Ok(data) => handle_send_message_data(data, state, sender).await,
        Err(e) => Err(format!("failed to parse websocket message: {e}")),
    }
}

async fn handle_send_message_data(
    msg: WebSocketSendMsg,
    state: Arc<InternalState>,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<(), String> {
    if msg.cmd == "sendmsg" {
        let hopr = state.hopr.clone();

        // Use the message encoder, if any
        // TODO: remove RLP in 3.0
        let msg_body = state
            .msg_encoder
            .as_ref()
            .map(|enc| enc(&msg.args.body))
            .unwrap_or_else(|| msg.args.body.into_boxed_slice());

        let options = if let Some(intermediate_path) = msg.args.path {
            PathOptions::IntermediatePath(intermediate_path)
        } else if let Some(hops) = msg.args.hops {
            PathOptions::Hops(hops)
        } else {
            return Err("one of hops or intermediate path must be provided".to_string());
        };

        let hkc = hopr
            .send_message(msg_body, msg.args.peer_id, options, Some(msg.args.tag))
            .await;

        match hkc {
            Ok(challenge) => {
                if let Err(e) = sender
                    .send(Message::Text(
                        json!(WebSocketReadAck::from_ack_challenge(challenge)).to_string(),
                    ))
                    .await
                {
                    return Err(format!("failed to send websocket message: {e}"));
                }
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
    } else {
        warn!("skipping an unsupported websocket command '{}'", msg.cmd);
    }

    Ok(())
}

/// Delete messages from nodes message inbox.
#[utoipa::path(
        delete,
        path = const_format::formatcp!("{BASE_PATH}/messages"),
        params(TagQueryRequest),
        responses(
            (status = 204, description = "Messages successfully deleted."),
            (status = 400, description = "Bad request.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        tag = "Messages",
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        )
    )]
pub(super) async fn delete_messages(
    Query(TagQueryRequest { tag }): Query<TagQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    if let Some(tag) = tag {
        if tag < RESERVED_TAG_UPPER_LIMIT {
            return (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidApplicationTag).into_response();
        }
    }

    let inbox = state.inbox.clone();
    inbox.write().await.pop_all(tag).await;
    (StatusCode::NO_CONTENT, ()).into_response()
}

/// Get size of filtered message inbox for a specific tag
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/messages/size"),
        params(TagQueryRequest),
        responses(
            (status = 200, description = "Returns the message inbox size filtered by the given tag", body = SizeResponse),
            (status = 400, description = "Bad request.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Messages"
    )]
pub(super) async fn size(
    Query(TagQueryRequest { tag }): Query<TagQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    if let Some(tag) = tag {
        if tag < RESERVED_TAG_UPPER_LIMIT {
            return (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidApplicationTag).into_response();
        }
    }

    let inbox = state.inbox.clone();
    let size = inbox.read().await.size(tag).await;

    (StatusCode::OK, Json(SizeResponse { size })).into_response()
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "body": "Test message 1",
        "receivedAt": 1704453953073i64,
        "tag": 2000
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
            (status = 400, description = "Bad request.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Messages"
    )]
pub(super) async fn pop(
    Query(TagQueryRequest { tag }): Query<TagQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    if let Some(tag) = tag {
        if tag < RESERVED_TAG_UPPER_LIMIT {
            return (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidApplicationTag).into_response();
        }
    }

    let inbox = state.inbox.clone();
    let inbox = inbox.write().await;
    if let Some((data, ts)) = inbox.pop(tag).await {
        match to_api_message(data, ts) {
            Ok(message) => (StatusCode::OK, Json(message)).into_response(),
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::UnknownFailure(e)).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, ()).into_response()
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub(crate) struct MessagePopAllResponse {
    messages: Vec<MessagePopResponse>,
}

/// Get the list of messages currently present in the nodes message inbox.
///
/// The messages are removed from the inbox.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/messages/pop-all"),
        request_body(
            content = TagQueryRequest,
            description = "Tag of message queue to pop from. When an empty object or an object with a `tag: 0` is provided, it lists and removes all the messages.",
            content_type = "application/json"
        ),
        responses(
            (status = 200, description = "All message successfully extracted.", body = MessagePopAllResponse),
            (status = 400, description = "Bad request.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Messages"
    )]
pub(super) async fn pop_all(
    Query(TagQueryRequest { tag }): Query<TagQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    if let Some(tag) = tag {
        if tag < RESERVED_TAG_UPPER_LIMIT {
            return (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidApplicationTag).into_response();
        }
    }

    let inbox = state.inbox.clone();
    let inbox = inbox.write().await;
    let messages: Vec<MessagePopResponse> = inbox
        .pop_all(tag)
        .await
        .into_iter()
        .filter_map(|(data, ts)| match to_api_message(data, ts) {
            Ok(msg) => Some(msg),
            Err(e) => {
                error!("failed to pop message: {e}");
                None
            }
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(MessagePopAllResponse { messages })).into_response()
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
            (status = 400, description = "Bad request.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Messages"
    )]
pub(super) async fn peek(
    Query(TagQueryRequest { tag }): Query<TagQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> impl IntoResponse {
    if let Some(tag) = tag {
        if tag < RESERVED_TAG_UPPER_LIMIT {
            return (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidApplicationTag).into_response();
        }
    }

    let inbox = state.inbox.clone();
    let inbox = inbox.write().await;
    if let Some((data, ts)) = inbox.peek(tag).await {
        match to_api_message(data, ts) {
            Ok(message) => (StatusCode::OK, Json(message)).into_response(),
            Err(e) => (StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::UnknownFailure(e)).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, ()).into_response()
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
            description = "Tag of message queue and optionally a timestamp since from to start peeking. When an empty object or an object with a `tag: 0` is provided, it fetches all the messages.",
            content_type = "application/json"
        ),
        responses(
            (status = 200, description = "All messages successfully peeked at.", body = MessagePopAllResponse),
            (status = 400, description = "Bad request.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "The specified resource was not found."),
            (status = 422, description = "Unknown failure", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Messages"
    )]

pub(super) async fn peek_all(
    State(state): State<Arc<InternalState>>,
    Json(GetMessageBodyRequest { tag, timestamp }): Json<GetMessageBodyRequest>,
) -> impl IntoResponse {
    if let Some(tag) = tag {
        if tag < RESERVED_TAG_UPPER_LIMIT {
            return (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidApplicationTag).into_response();
        }
    }

    let inbox = state.inbox.clone();
    let inbox = inbox.read().await;
    let messages = inbox
        .peek_all(tag, timestamp)
        .await
        .into_iter()
        .filter_map(|(data, ts)| match to_api_message(data, ts) {
            Ok(msg) => Some(msg),
            Err(e) => {
                error!("failed to peek message: {e}");
                None
            }
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(MessagePopAllResponse { messages })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::from_value;

    #[test]
    fn send_message_accepts_bytes_in_body() {
        let peer = PeerId::random();
        let test_sequence = b"wow, this actually works";

        let json_value = json!({
            "tag": 5,
            "body": test_sequence.to_vec(),
            "peerId": peer.to_string()
        });

        let expected = SendMessageBodyRequest {
            tag: 5,
            body: test_sequence.to_vec(),
            peer_id: peer,
            path: None,
            hops: None,
        };

        let actual: SendMessageBodyRequest = from_value(json_value).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn send_message_accepts_utf8_string_in_body() {
        let peer = PeerId::random();
        let test_sequence = b"wow, this actually works";

        let json_value = json!({
            "tag": 5,
            "body": String::from_utf8(test_sequence.to_vec()).expect("should be a utf-8 string"),
            "peerId": peer.to_string()
        });

        let expected = SendMessageBodyRequest {
            tag: 5,
            body: test_sequence.to_vec(),
            peer_id: peer,
            path: None,
            hops: None,
        };

        let actual: SendMessageBodyRequest = from_value(json_value).unwrap();

        assert_eq!(actual, expected);
    }
}
