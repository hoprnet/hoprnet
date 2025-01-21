use axum::{
    extract::{Json, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_crypto_types::types::Hash;
use serde::Deserialize;
use serde_with::{serde_as, Bytes, DisplayFromStr, DurationMilliSeconds};
use std::{sync::Arc, time::Duration};
use tracing::error;
use validator::Validate;

use hopr_lib::{
    errors::{HoprLibError, HoprStatusError},
    AsUnixTimestamp, RoutingOptions, RESERVED_TAG_UPPER_LIMIT,
};

use crate::{
    types::{HoprIdentifier, PeerOrAddress},
    ApiError, ApiErrorStatus, InternalState, BASE_PATH,
};

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
        "path": [
            "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33"
        ],
        "destination": "12D3KooWEDc1vGJevww48trVDDf6pr1f6N3F86sGJfQrKCyc8kJ1",
        "tag": 2000
    }))]
#[schema(example = json!({
    "body": "Test message",
    "hops": 1,
    "peerId": "12D3KooWEDc1vGJevww48trVDDf6pr1f6N3F86sGJfQrKCyc8kJ1",
    "tag": 2000
}))]
pub(crate) struct SendMessageBodyRequest {
    /// The message tag used to filter messages based on application, must be from range <1024,65535>
    #[schema(minimum = 1024, maximum = 65535)]
    tag: u16,
    /// Message to be transmitted over the network
    #[serde_as(as = "Bytes")]
    body: Vec<u8>,
    /// The recipient HOPR PeerId or address
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    destination: PeerOrAddress,
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    #[validate(length(min = 0, max = 3))]
    #[schema(value_type = Option<Vec<String>>)]
    path: Option<Vec<PeerOrAddress>>,
    #[schema(minimum = 0, maximum = 3)]
    #[validate(range(min = 0, max = 3))]
    hops: Option<u16>,
}

#[serde_as]
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[schema(example = json!({
        "timestamp": 2147483647
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SendMessageResponse {
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    #[schema(value_type = u64)]
    timestamp: std::time::Duration,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    challenge: Hash,
}

#[serde_as]
#[derive(Debug, Default, Clone, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
    "tag": 801,
    "timestamp": 2147483647
}))]
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
            (status = 412, description = "The node is not ready."),
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
) -> Result<impl IntoResponse, impl IntoResponse> {
    let hopr = state.hopr.clone();

    args.validate().map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
            .into_response()
    })?;

    let peer_id = match HoprIdentifier::new_with(args.destination, hopr.peer_resolver()).await {
        Ok(destination) => destination.peer_id,
        Err(e) => return Err(e.into_response()),
    };

    #[cfg(not(feature = "explicit-path"))]
    if args.path.is_some() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::InvalidPath("explicit paths are not allowed".into()),
        )
            .into_response());
    }

    let options = if let Some(intermediate_path) = args.path {
        let peer_ids_future = intermediate_path
            .into_iter()
            .map(|address| HoprIdentifier::new_with(address, hopr.peer_resolver()))
            .collect::<Vec<_>>();

        let path = futures::future::try_join_all(peer_ids_future)
            .await
            .map_err(|e: ApiErrorStatus| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(format!("Failed to fulfill path: {e}")),
                )
                    .into_response()
            })?
            .into_iter()
            .map(|v| v.peer_id)
            .collect::<Vec<_>>();

        RoutingOptions::IntermediatePath(
            // get a vec of peer ids from the intermediate path
            path.try_into().map_err(|_| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::InvalidPath("Invalid number of hops".into()),
                )
                    .into_response()
            })?,
        )
    } else if let Some(hops) = args.hops {
        RoutingOptions::Hops((hops as u8).try_into().map_err(|_| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::InvalidPath(format!(
                    "Number of hops cannot be larger than {}",
                    RoutingOptions::MAX_INTERMEDIATE_HOPS
                )),
            )
                .into_response()
        })?)
    } else {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::InvalidPath("One of either hops or intermediate path must be specified".into()),
        )
            .into_response());
    };

    let timestamp = std::time::SystemTime::now().as_unix_timestamp();

    match hopr
        .send_message(args.body.into_boxed_slice(), peer_id, options, Some(args.tag))
        .await
    {
        Ok(_) => Ok((
            StatusCode::ACCEPTED,
            Json(SendMessageResponse {
                timestamp,
                challenge: Hash::create(&[b"This value is useless and is present only for backwards compatibility"]),
            }),
        )
            .into_response()),
        Err(HoprLibError::StatusError(HoprStatusError::NotThereYet(_, _))) => {
            Err((StatusCode::PRECONDITION_FAILED, ApiErrorStatus::NotReady).into_response())
        }
        Err(e) => Err((StatusCode::UNPROCESSABLE_ENTITY, ApiErrorStatus::from(e)).into_response()),
    }
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
    State(state): State<Arc<InternalState>>,
    Json(TagQueryRequest { tag }): Json<TagQueryRequest>,
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
    State(state): State<Arc<InternalState>>,
    Json(TagQueryRequest { tag }): Json<TagQueryRequest>,
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
                error!(error = %e, "failed to pop message");
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
    State(state): State<Arc<InternalState>>,
    Json(TagQueryRequest { tag }): Json<TagQueryRequest>,
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
                error!(error = %e, "failed to peek message:");
                None
            }
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(MessagePopAllResponse { messages })).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    use libp2p_identity::PeerId;
    use serde_json::{from_value, json};

    #[test]
    fn send_message_accepts_bytes_in_body() -> anyhow::Result<()> {
        let peer_id = PeerId::random();
        let destination = PeerOrAddress::from(peer_id);
        let test_sequence = b"wow, this actually works";

        let json_value = json!({
            "tag": 5,
            "body": test_sequence.to_vec(),
            "destination": peer_id,
        });

        let expected = SendMessageBodyRequest {
            tag: 5,
            body: test_sequence.to_vec(),
            destination: destination,
            path: None,
            hops: None,
        };

        let actual: SendMessageBodyRequest = from_value(json_value)?;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn send_message_accepts_utf8_string_in_body() -> anyhow::Result<()> {
        let peer_id = PeerId::random();
        let destination = PeerOrAddress::from(peer_id);
        let test_sequence = b"wow, this actually works";

        let json_value = json!({
            "tag": 5,
            "destination": peer_id,
            "body": String::from_utf8(test_sequence.to_vec())?,
        });

        let expected = SendMessageBodyRequest {
            tag: 5,
            body: test_sequence.to_vec(),
            destination: destination,
            path: None,
            hops: None,
        };

        let actual: SendMessageBodyRequest = from_value(json_value)?;

        assert_eq!(actual, expected);

        Ok(())
    }
}
