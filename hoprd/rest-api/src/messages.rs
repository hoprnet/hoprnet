use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_crypto_types::types::Hash;
use serde::Deserialize;
use serde_with::{serde_as, Bytes, DisplayFromStr, DurationMilliSeconds};
use std::sync::Arc;
use validator::Validate;

use crate::{
    types::{HoprIdentifier, PeerOrAddress},
    ApiError, ApiErrorStatus, InternalState, BASE_PATH,
};
use hopr_lib::{
    errors::{HoprLibError, HoprStatusError},
    AsUnixTimestamp, RoutingOptions,
};
use hopr_network_types::prelude::DestinationRouting;

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

    let peer_addr = match HoprIdentifier::new_with(args.destination, hopr.peer_resolver()).await {
        Ok(destination) => destination.address,
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
            .map(|v| v.address)
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

    // NOTE: The return path is not introduced to, because the send_message API will be deprecated
    let routing = DestinationRouting::Forward {
        destination: peer_addr,
        pseudonym: None,
        forward_options: options,
        return_options: None,
    };
    match hopr.send_message(args.body.into_boxed_slice(), routing, args.tag).await {
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
