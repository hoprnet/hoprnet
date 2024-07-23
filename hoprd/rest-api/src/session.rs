use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use hopr_lib::{PathOptions, PeerId, SessionCapability, SessionClientConfig};
use tokio::{io::copy_bidirectional_with_sizes, net::TcpListener};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{error, info};

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

pub const SESSION_TO_SOCKET_BUFEER: usize = 1456;
pub const SOCKET_TO_SESSION_BUFEER: usize = 1456;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "destination": "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33",
        "path": {
            "Hops": 1
        },
        "port": 0
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub destination: PeerId,
    pub path: PathOptions,
    #[serde(default)]
    pub port: u16,
}

impl From<SessionClientRequest> for SessionClientConfig {
    fn from(value: SessionClientRequest) -> Self {
        Self {
            peer: value.destination,
            path_options: value.path,
            capabilities: vec![], //vec![SessionCapability::Retransmission, SessionCapability::Segmentation],
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "port": 5542,
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientResponse {
    pub port: u16,
}

/// Creates a new client session returing a dedicated session listening port.
///
/// Once the port is bound, it is possible to use the socket for bidirectional read and write communication.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/session"),
        request_body(
            content = SessionClientRequest,
            description = "Creates a new client HOPR session that will start listening on a dedicated port. Once the port is bound, it is possible to use the socketfor bidirectional read and write communication.",
            content_type = "application/json"),
        responses(
            (status = 200, description = "Successfully created a new client session", body = SessionClientResponse),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError),
            (status = 501, description = "Feature not implemented", body = ApiError)
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Session"
    )]
pub(crate) async fn create_client(
    State(state): State<Arc<InternalState>>,
    Json(args): Json<SessionClientRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let port = args.port;
    let data: SessionClientConfig = args.into();
    // let is_tcp_like = data.capabilities.contains(&SessionCapability::Retransmission)
    //     || data.capabilities.contains(&SessionCapability::Segmentation);

    let is_tcp_like = true;

    if is_tcp_like {
        let session = state.hopr.clone().connect_to(data).await.map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::UnknownFailure(e.to_string()),
            )
        })?;

        let tcp_listener = TcpListener::bind(format!("127.0.0.1:{port}")).await.map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::UnknownFailure(format!("Failed to bind on 127.0.0.1:{port}: {e}")),
            )
        })?;

        let port = tcp_listener
            .local_addr()
            .map_err(|e| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(format!("Failed to get the port number: {e}")),
                )
            })?
            .port();

        tokio::task::spawn(async move {
            match tcp_listener
            .accept()
            .await {
                Ok((mut tcp_stream, _sock_addr)) => match copy_bidirectional_with_sizes(&mut session.compat(), &mut tcp_stream, SESSION_TO_SOCKET_BUFEER, SOCKET_TO_SESSION_BUFEER).await {
                    Ok(bound_stream_finished) => info!("Client session through TCP port {port} ended with {bound_stream_finished:?} bytes transferred in both directions."),
                    Err(e) => error!("Failed to bind the TCP stream (port {port}) to the session: {e}")
                },
                Err(e) => error!("Failed to accept connection: {e}")
            }
        });

        Ok((StatusCode::OK, Json(SessionClientResponse { port })).into_response())
    } else {
        // let s = UdpSocket::bind("0.0.0.0:{port}").await?.connect().await;
        Err((
            StatusCode::NOT_IMPLEMENTED,
            ApiErrorStatus::UnknownFailure("No UDP socket support yet".to_string()),
        ))
    }
}
