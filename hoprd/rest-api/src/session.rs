use std::sync::Arc;

use crate::{ApiErrorStatus, InternalState, BASE_PATH};
use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use hopr_lib::errors::HoprLibError;
use hopr_lib::{HoprSession, IpProtocol, PeerId, RoutingOptions, SessionCapability, SessionClientConfig};
use hopr_network_types::prelude::ConnectedUdpStream;
use hopr_network_types::udp::ForeignDataMode;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{debug, error, info};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "destination": "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33",
        "path": {
            "Hops": 1
        },
        "protocol": "TCP",
        "target": "localhost:8080",
        "listen_port": 10000,
        "capabilities": ["Retransmission", "Segmentation"]
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub destination: PeerId,
    pub path: RoutingOptions,
    pub protocol: IpProtocol,
    pub target: String,
    pub listen_port: Option<u16>,
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    pub capabilities: Option<Vec<SessionCapability>>,
}

impl TryFrom<SessionClientRequest> for SessionClientConfig {
    type Error = HoprLibError;

    fn try_from(value: SessionClientRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            peer: value.destination,
            path_options: value.path,
            target_protocol: value.protocol,
            target: value
                .target
                .parse()
                .map_err(|e| HoprLibError::GeneralError(format!("target host parse error: {e}")))?,
            capabilities: value.capabilities.unwrap_or_else(|| match value.protocol {
                IpProtocol::TCP => {
                    vec![SessionCapability::Retransmission, SessionCapability::Segmentation]
                }
                _ => vec![], // no default capabilities for UDP, etc.
            }),
        })
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

/// Creates a new client session returning the given session listening port over TCP or UDP.
/// If no listening port is given in the request, the socket will be bound to a random free
/// port and returned in the response.
/// Different capabilities can be configured for the session, such as data segmentation or
/// retransmission.
///
/// Once the port is bound, it is possible to use the socket for bidirectional read/write
/// communication over the selected IP protocol and HOPR network routing with the given destination.
/// The destination HOPR node forwards all the data to the given target over the selected IP protocol.
///
/// Various services require different types of socket communications:
/// - services running over UDP usually do not require data retransmission, as it is already expected
/// that UDP does not provide these and is therefore handled at the application layer.
/// - On the contrary, services running over TCP *almost always* expect data segmentation and
/// retransmission capabilities, so these should be configured while creating a session that passes
/// TCP data.
#[utoipa::path(
        post,
        path = const_format::formatcp!("{BASE_PATH}/session"),
        request_body(
            content = SessionClientRequest,
            description = "Creates a new client HOPR session that will start listening on a dedicated port. Once the port is bound, it is possible to use the socket for bidirectional read and write communication.",
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
    let port = args.listen_port.unwrap_or(0); // if not given, listen on random port
    let data = SessionClientConfig::try_from(args).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
    })?;
    let protocol = data.target_protocol;

    let session = state.hopr.clone().connect_to(data).await.map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
    })?;

    match protocol {
        IpProtocol::TCP => {
            let (port, tcp_listener) = tcp_listen_on(("127.0.0.1", port)).await.map_err(|e| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(format!("Failed to start TCP listener on 127.0.0.1:{port}: {e}")),
                )
            })?;

            tokio::task::spawn(async move {
                match tcp_listener.accept().await {
                    Ok((stream, sock_addr)) => {
                        debug!(
                            session_id = tracing::field::debug(*session.id()),
                            "accepted incoming TCP connection from {sock_addr}"
                        );
                        bind_session_to_stream(session, stream).await
                    }
                    Err(e) => error!(
                        session_id = tracing::field::debug(*session.id()),
                        "failed to accept connection: {e}"
                    ),
                }
            });

            Ok::<_, (StatusCode, ApiErrorStatus)>(
                (StatusCode::OK, Json(SessionClientResponse { port })).into_response(),
            )
        }
        IpProtocol::UDP => {
            let (port, udp_socket) = udp_bind_to(("127.0.0.1", port)).await.map_err(|e| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(format!("Failed to start UDP listener on 127.0.0.1:{port}: {e}")),
                )
            })?;

            tokio::task::spawn(bind_session_to_stream(session, udp_socket));

            Ok::<_, (StatusCode, ApiErrorStatus)>(
                (StatusCode::OK, Json(SessionClientResponse { port })).into_response(),
            )
        }
    }
}

async fn tcp_listen_on<A: ToSocketAddrs>(address: A) -> std::io::Result<(u16, TcpListener)> {
    let tcp_listener = TcpListener::bind(address).await?;

    Ok((tcp_listener.local_addr()?.port(), tcp_listener))
}

async fn udp_bind_to<A: ToSocketAddrs>(address: A) -> std::io::Result<(u16, ConnectedUdpStream)> {
    let udp_socket = ConnectedUdpStream::bind(address)
        .await?
        .with_foreign_data_mode(ForeignDataMode::Discard); // discard data from UDP clients other than the first one served

    Ok((udp_socket.socket().local_addr()?.port(), udp_socket))
}

async fn bind_session_to_stream<T>(session: HoprSession, mut stream: T)
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let session_id = *session.id();
    match tokio::io::copy_bidirectional_with_sizes(
        &mut session.compat(),
        &mut stream,
        hopr_lib::SESSION_USABLE_MTU_SIZE,
        hopr_lib::SESSION_USABLE_MTU_SIZE,
    )
    .await
    {
        Ok(bound_stream_finished) => info!(
            "Client session {session_id} ended with {bound_stream_finished:?} bytes transferred in both directions.",
        ),
        Err(e) => error!("Failed to bind the stream to session {session_id}: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
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
    async fn hoprd_session_connection_should_create_a_working_tcp_socket_through_which_data_can_be_sent_and_received(
    ) -> anyhow::Result<()> {
        let (tx, rx) = futures::channel::mpsc::unbounded::<Box<[u8]>>();

        let peer: hopr_lib::PeerId = hopr_lib::HoprOffchainKeypair::random().public().into();
        let session = hopr_lib::HoprSession::new(
            hopr_lib::HoprSessionId::new(4567, peer),
            peer,
            RoutingOptions::IntermediatePath(Default::default()),
            HashSet::default(),
            Arc::new(SendMsgResender::new(tx)),
            rx,
            None,
        );

        let (port, tcp_listener) = tcp_listen_on(("127.0.0.1", 0)).await.context("listen_on failed")?;

        tokio::task::spawn(async move {
            match tcp_listener.accept().await {
                Ok((stream, _)) => bind_session_to_stream(session, stream).await,
                Err(e) => error!("failed to accept connection: {e}"),
            }
        });

        let mut tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .context("connect failed")?;

        let data = vec![b"hello", b"world", b"this ", b"is   ", b"    a", b" test"];

        for d in data.clone().into_iter() {
            tcp_stream.write_all(d).await.context("write failed")?;
        }

        for d in data.iter() {
            let mut buf = vec![0; d.len()];
            tcp_stream.read_exact(&mut buf).await.context("read failed")?;
        }

        Ok(())
    }

    #[tokio::test]
    async fn hoprd_session_connection_should_create_a_working_udp_socket_through_which_data_can_be_sent_and_received(
    ) -> anyhow::Result<()> {
        let (tx, rx) = futures::channel::mpsc::unbounded::<Box<[u8]>>();

        let peer: hopr_lib::PeerId = hopr_lib::HoprOffchainKeypair::random().public().into();
        let session = hopr_lib::HoprSession::new(
            hopr_lib::HoprSessionId::new(4567, peer),
            peer,
            RoutingOptions::IntermediatePath(Default::default()),
            HashSet::default(),
            Arc::new(SendMsgResender::new(tx)),
            rx,
            None,
        );

        let (_, udp_listener) = udp_bind_to(("127.0.0.1", 0)).await.context("udp_bind_to failed")?;

        let listen_addr = udp_listener.socket().local_addr()?;
        tokio::task::spawn(bind_session_to_stream(session, udp_listener));

        let mut tcp_stream = ConnectedUdpStream::bind(("127.0.0.1", 0))
            .await
            .context("bind failed")?
            .with_counterparty(listen_addr)?;

        let data = vec![b"hello", b"world", b"this ", b"is   ", b"    a", b" test"];

        for d in data.clone().into_iter() {
            tcp_stream.write_all(d).await.context("write failed")?;
        }

        for d in data.iter() {
            let mut buf = vec![0; d.len()];
            tcp_stream.read_exact(&mut buf).await.context("read failed")?;
        }

        Ok(())
    }
}
