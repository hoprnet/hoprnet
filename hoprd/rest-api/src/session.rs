use std::sync::Arc;

use axum::{
    extract::{Json, Path, Query, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use hopr_lib::{PathOptions, PeerId, SessionCapability, SessionClientConfig};

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "destination": "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33",
        "path": {
            "Hops":1
        },
        "capabilities": ["Segmentation", "Retransmission"]
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub destination: PeerId,
    pub path: PathOptions,
    pub capabilities: Vec<SessionCapability>,
    pub port: Option<u16>,
}

impl From<SessionClientRequest> for SessionClientConfig {
    fn from(value: SessionClientRequest) -> Self {
        Self {
            peer: value.destination,
            path_options: value.path,
            capabilities: value.capabilities,
        }
    }
}

/// Creates a new client session returing a dedicated session listening port.
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/messages/session"),
        request_body(
            content = SessionClientRequest,
            description = "Configuration of the ",
            content_type = "application/json"),
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
pub async fn session(
    State(state): State<Arc<InternalState>>,
    Json(args): Json<SessionClientRequest>,
) -> impl IntoResponse {
    let port = args.port;
    let data: SessionClientConfig = args.into();

    let inbox = state.inbox.clone();

    // let inbox = inbox.write().await;
    // if let Some((data, ts)) = inbox.peek(tag.tag).await {
    //     match to_api_message(data, ts) {
    //         Ok(message) => Ok(Response::builder(200).body(json!(message)).build()),
    //         Err(e) => Ok(Response::builder(422).body(ApiErrorStatus::UnknownFailure(e)).build()),
    //     }
    // } else {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        ApiErrorStatus::UnknownFailure("REPLACE THIS".to_owned()),
    )
        .into_response()
    // }
}
