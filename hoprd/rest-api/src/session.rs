use std::sync::Arc;

use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use hopr_lib::{HoprSession, IpProtocol, PeerId, RoutingOptions, SessionClientConfig};
use tokio::net::TcpListener;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{error, info};

use crate::{ApiErrorStatus, InternalState, BASE_PATH};

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
    pub path: RoutingOptions,
    #[serde(default)]
    pub port: u16,
}

impl From<SessionClientRequest> for SessionClientConfig {
    fn from(value: SessionClientRequest) -> Self {
        Self {
            peer: value.destination,
            path_options: value.path,
            target_protocol: IpProtocol::TCP,
            target: "127.0.0.1:3000".parse().unwrap(),
            capabilities: vec![],
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
/// Various services require diffrent types of socket communications. This is set by the capabilities field.
///
/// TODO: The prototype implementation does not support UDP sockets yet and forces the usage of a TCP socket.
/// Such a restriction is not ideal and should be removed in the future.
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

        let (port, tcp_listener) = listen_on(format!("127.0.0.1:{port}")).await.map_err(|e| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::UnknownFailure(format!("Failed to bind on 127.0.0.1:{port}: {e}")),
            )
        })?;

        tokio::task::spawn(bind_session_to_connection(session, tcp_listener));

        Ok((StatusCode::OK, Json(SessionClientResponse { port })).into_response())
    } else {
        // let s = UdpSocket::bind("0.0.0.0:{port}").await?.connect().await;
        Err((
            StatusCode::NOT_IMPLEMENTED,
            ApiErrorStatus::UnknownFailure("No UDP socket support yet".to_string()),
        ))
    }
}

async fn listen_on(address: String) -> std::io::Result<(u16, TcpListener)> {
    let tcp_listener = TcpListener::bind(address).await?;

    Ok((tcp_listener.local_addr()?.port(), tcp_listener))
}

async fn bind_session_to_connection(session: HoprSession, tcp_listener: TcpListener) {
    let session_id = *session.id();
    match tcp_listener.accept().await {
        Ok((mut tcp_stream, _sock_addr)) => match tokio::io::copy_bidirectional_with_sizes(&mut session.compat(), &mut tcp_stream, hopr_lib::SESSION_USABLE_MTU_SIZE, hopr_lib::SESSION_USABLE_MTU_SIZE).await {
            Ok(bound_stream_finished) => info!(
                "Client session {session_id} ended with {bound_stream_finished:?} bytes transferred in both directions.",
            ),
            Err(e) => error!("Failed to bind the TCP stream to session {session_id}: {e}"),
        },
        Err(e) => error!("Failed to accept connection: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::mpsc::UnboundedSender;
    use hopr_lib::{ApplicationData, Keypair, PeerId, SendMsg};
    use hopr_transport_session::errors::TransportSessionError;
    use hopr_transport_session::RoutingOptions;
    use std::collections::HashSet;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    pub struct SendMsgResender {
        tx: UnboundedSender<Box<[u8]>>,
    }

    impl SendMsgResender {
        pub fn new(tx: UnboundedSender<Box<[u8]>>) -> Self {
            Self { tx }
        }
    }

    #[hopr_lib::async_trait]
    impl SendMsg for SendMsgResender {
        // Mimics the echo server by feeding the data back in instead of sending it over the wire
        async fn send_message(
            &self,
            data: ApplicationData,
            _destination: PeerId,
            _options: RoutingOptions,
        ) -> std::result::Result<(), TransportSessionError> {
            let (_peer, data) = hopr_transport_session::types::unwrap_offchain_key(data.plain_text)?;

            self.tx
                .clone()
                .unbounded_send(data)
                .expect("send message: failed to send data");

            Ok(())
        }
    }

    #[tokio::test]
    async fn hoprd_session_connection_should_create_a_working_socket_through_which_data_can_be_sent_and_received() {
        let (tx, rx) = futures::channel::mpsc::unbounded::<Box<[u8]>>();

        let peer: hopr_lib::PeerId = hopr_lib::HoprOffchainKeypair::random().public().into();
        let session = hopr_lib::HoprSession::new(
            hopr_lib::HoprSessionId::new(4567, peer),
            peer,
            hopr_lib::RoutingOptions::IntermediatePath(Default::default()),
            HashSet::default(),
            Arc::new(SendMsgResender::new(tx)),
            rx,
        );

        let (port, tcp_listener) = listen_on(format!("127.0.0.1:0")).await.expect("listen_on succeeded");
        tokio::task::spawn(bind_session_to_connection(session, tcp_listener));

        let mut tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();

        let data = vec![b"hello", b"world", b"this ", b"is   ", b"    a", b" test"];

        for d in data.clone().into_iter() {
            tcp_stream.write_all(d).await.unwrap();
        }

        for d in data.iter() {
            let mut buf = vec![0; d.len()];
            tcp_stream.read_exact(&mut buf).await.unwrap();
        }
    }
}
