use std::io::ErrorKind;
use std::str::FromStr;
use std::sync::Arc;

use crate::{ApiErrorStatus, InternalState, ListenerId, BASE_PATH};
use axum::extract::Path;
use axum::{
    extract::{Json, State},
    http::status::StatusCode,
    response::IntoResponse,
};
use futures::{StreamExt, TryStreamExt};
use hopr_lib::errors::HoprLibError;
use hopr_lib::{HoprSession, IpProtocol, PeerId, RoutingOptions, SessionCapability, SessionClientConfig};
use hopr_network_types::prelude::ConnectedUdpStream;
use hopr_network_types::udp::ForeignDataMode;
use hopr_network_types::utils::copy_duplex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{debug, error, info};

/// Default listening host the session listener socket binds to.
pub const DEFAULT_LISTEN_HOST: &str = "127.0.0.1:0";

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "destination": "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33",
        "path": {
            "Hops": 1
        },
        "protocol": "TCP",
        "target": "localhost:8080",
        "listen_host": "127.0.0.1:10000",
        "capabilities": ["Retransmission", "Segmentation"]
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientRequest {
    #[serde_as(as = "DisplayFromStr")]
    pub destination: PeerId,
    pub path: RoutingOptions,
    pub target: String,
    pub listen_host: Option<String>,
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    pub capabilities: Option<Vec<SessionCapability>>,
}

impl SessionClientRequest {
    pub(crate) fn into_protocol_session_config(
        self,
        target_protocol: IpProtocol,
    ) -> Result<SessionClientConfig, HoprLibError> {
        Ok(SessionClientConfig {
            peer: self.destination,
            path_options: self.path,
            target_protocol,
            target: self
                .target
                .parse()
                .map_err(|e| HoprLibError::GeneralError(format!("target host parse error: {e}")))?,
            capabilities: self.capabilities.unwrap_or_else(|| match target_protocol {
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
        "protocol": "tcp",
        "ip": "127.0.0.1",
        "port": 5542
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientResponse {
    #[serde_as(as = "DisplayFromStr")]
    pub protocol: IpProtocol,
    pub ip: String,
    pub port: u16,
}

/// Creates a new client session returning the given session listening host & port over TCP or UDP.
/// If no listening port is given in the request, the socket will be bound to a random free
/// port and returned in the response.
/// Different capabilities can be configured for the session, such as data segmentation or
/// retransmission.
///
/// Once the host and port are bound, it is possible to use the socket for bidirectional read/write
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
        path = const_format::formatcp!("{BASE_PATH}/session/{{protocol}}"),
        request_body(
            content = SessionClientRequest,
            description = "Creates a new client HOPR session that will start listening on a dedicated port. Once the port is bound, it is possible to use the socket for bidirectional read and write communication.",
            content_type = "application/json"),
        responses(
            (status = 200, description = "Successfully created a new client session", body = SessionClientResponse),
            (status = 400, description = "Invalid IP protocol.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 409, description = "Listening address and port are already used.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Session"
    )]
pub(crate) async fn create_client(
    State(state): State<Arc<InternalState>>,
    Path(protocol): Path<IpProtocol>,
    Json(args): Json<SessionClientRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let bind_host: std::net::SocketAddr = args
        .listen_host
        .clone()
        .unwrap_or(DEFAULT_LISTEN_HOST.to_string())
        .parse()
        .map_err(|_| {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorStatus::UnknownFailure("invalid listening host".into()),
            )
        })?;

    if bind_host.port() > 0
        && state
            .open_listeners
            .read()
            .await
            .contains_key(&ListenerId(protocol, bind_host))
    {
        return Err((StatusCode::CONFLICT, ApiErrorStatus::InvalidInput));
    }

    let data = args.into_protocol_session_config(protocol).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
    })?;

    let bound_host = match protocol {
        IpProtocol::TCP => {
            let (bound_host, tcp_listener) = tcp_listen_on(bind_host).await.map_err(|e| {
                if e.kind() == ErrorKind::AddrInUse {
                    (StatusCode::CONFLICT, ApiErrorStatus::InvalidInput)
                } else {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ApiErrorStatus::UnknownFailure(format!("failed to start TCP listener on {bind_host}: {e}")),
                    )
                }
            })?;

            let hopr = state.hopr.clone();
            let jh = hopr_async_runtime::prelude::spawn(
                tokio_stream::wrappers::TcpListenerStream::new(tcp_listener)
                    .and_then(|sock| async { Ok((sock.peer_addr()?, sock)) })
                    .for_each_concurrent(None, move |accepted_client| {
                        let data = data.clone();
                        let hopr = hopr.clone();
                        async move {
                            match accepted_client {
                                Ok((sock_addr, stream)) => {
                                    debug!("incoming TCP connection {sock_addr}");
                                    let session = match hopr.connect_to(data).await {
                                        Ok(s) => s,
                                        Err(e) => {
                                            error!("failed to establish session: {e}");
                                            return;
                                        }
                                    };

                                    debug!(
                                        session_id = tracing::field::debug(*session.id()),
                                        "new session for incoming TCP connection from {sock_addr}",
                                    );

                                    bind_session_to_stream(session, stream).await
                                }
                                Err(e) => error!("failed to accept connection: {e}"),
                            }
                        }
                    }),
            );

            state
                .open_listeners
                .write()
                .await
                .insert(ListenerId(protocol, bound_host), jh);
            bound_host
        }
        IpProtocol::UDP => {
            let session = state.hopr.clone().connect_to(data).await.map_err(|e| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(e.to_string()),
                )
            })?;

            let (bound_host, udp_socket) = udp_bind_to(bind_host).await.map_err(|e| {
                if e.kind() == ErrorKind::AddrInUse {
                    (
                        StatusCode::CONFLICT,
                        ApiErrorStatus::UnknownFailure(format!("cannot bind to: {bind_host}: {e}")),
                    )
                } else {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ApiErrorStatus::UnknownFailure(format!("failed to start UDP listener on {bind_host}: {e}")),
                    )
                }
            })?;

            state.open_listeners.write().await.insert(
                ListenerId(protocol, bound_host),
                hopr_async_runtime::prelude::spawn(bind_session_to_stream(session, udp_socket)),
            );
            bound_host
        }
    };

    Ok::<_, (StatusCode, ApiErrorStatus)>(
        (
            StatusCode::OK,
            Json(SessionClientResponse {
                protocol,
                ip: bound_host.ip().to_string(),
                port: bound_host.port(),
            }),
        )
            .into_response(),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "listeningIp": "127.0.0.1",
        "port": 5542
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionCloseClientRequest {
    pub listening_ip: String,
    pub port: u16,
}

/// Closes an existing Session listener.
/// The listener must've been previously created and bound for the given IP protocol.
/// Once a listener is closed, no more socket connections can be made to it.
#[utoipa::path(
    delete,
    path = const_format::formatcp!("{BASE_PATH}/session/{{protocol}}"),
    params(
            ("protocol" = String, Path, description = "IP protocol")
    ),
    request_body(
            content = SessionCloseClientRequest,
            description = "Closes the listener on the given bound IP address and port.",
            content_type = "application/json"),
    responses(
            (status = 204, description = "Listener closed successfully"),
            (status = 400, description = "Invalid IP protocol or port.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Listener not found.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
    ),
    security(
            ("api_token" = []),
            ("bearer_token" = [])
    ),
    tag = "Session",
)]
pub(crate) async fn close_client(
    State(state): State<Arc<InternalState>>,
    Path(protocol): Path<IpProtocol>,
    Json(SessionCloseClientRequest { listening_ip, port }): Json<SessionCloseClientRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let bound_addr = std::net::SocketAddr::from_str(&format!("{listening_ip}:{port}"))
        .map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidInput))?;

    let handle = state
        .open_listeners
        .write()
        .await
        .remove(&ListenerId(protocol, bound_addr))
        .ok_or((StatusCode::NOT_FOUND, ApiErrorStatus::InvalidInput))?;

    let _ = handle.cancel().await;
    Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::NO_CONTENT, "").into_response())
}

async fn tcp_listen_on<A: ToSocketAddrs>(address: A) -> std::io::Result<(std::net::SocketAddr, TcpListener)> {
    let tcp_listener = TcpListener::bind(address).await?;

    Ok((tcp_listener.local_addr()?, tcp_listener))
}

async fn udp_bind_to<A: ToSocketAddrs>(address: A) -> std::io::Result<(std::net::SocketAddr, ConnectedUdpStream)> {
    let udp_socket = ConnectedUdpStream::bind(address)
        .await?
        .with_foreign_data_mode(ForeignDataMode::Discard); // discard data from UDP clients other than the first one served

    Ok((udp_socket.socket().local_addr()?, udp_socket))
}

async fn bind_session_to_stream<T>(session: HoprSession, mut stream: T)
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let session_id = *session.id();
    match copy_duplex(
        &mut session.compat(),
        &mut stream,
        hopr_lib::SESSION_USABLE_MTU_SIZE,
        hopr_lib::SESSION_USABLE_MTU_SIZE,
    )
    .await
    {
        Ok(bound_stream_finished) => info!(
            session_id = tracing::field::debug(session_id),
            "client session ended with {bound_stream_finished:?} bytes transferred in both directions.",
        ),
        Err(e) => error!(
            session_id = tracing::field::debug(session_id),
            "error during data transfer: {e}"
        ),
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

        fn close(&self) {}
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

        let (bound_addr, tcp_listener) = tcp_listen_on(("127.0.0.1", 0)).await.context("listen_on failed")?;

        tokio::task::spawn(async move {
            match tcp_listener.accept().await {
                Ok((stream, _)) => bind_session_to_stream(session, stream).await,
                Err(e) => error!("failed to accept connection: {e}"),
            }
        });

        let mut tcp_stream = tokio::net::TcpStream::connect(bound_addr)
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

        let (listen_addr, udp_listener) = udp_bind_to(("127.0.0.1", 0)).await.context("udp_bind_to failed")?;

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
