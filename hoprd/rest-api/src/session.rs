use std::fmt::Formatter;
use std::future::Future;
use std::str::FromStr;

use axum::extract::Path;
use axum::Error;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, State,
    },
    http::status::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::Query;
use base64::Engine;
use futures::{AsyncReadExt, AsyncWriteExt, SinkExt, StreamExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::net::IpAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{debug, error, info, trace};

use crate::types::PeerOrAddress;
use crate::{ApiErrorStatus, InternalState, ListenerId, BASE_PATH};
use hopr_lib::errors::HoprLibError;
use hopr_lib::transfer_session;
use hopr_lib::{
    HoprSession, IpProtocol, RoutingOptions, ServiceId, SessionCapability, SessionClientConfig, SessionTarget,
};
use hopr_network_types::prelude::{ConnectedUdpStream, IpOrHost, SealedHost, UdpStreamParallelism};
use hopr_network_types::udp::ForeignDataMode;
use hopr_network_types::utils::AsyncReadStreamer;

/// Size of the buffer for forwarding data to/from a TCP stream.
pub const HOPR_TCP_BUFFER_SIZE: usize = 4096;

/// Size of the buffer for forwarding data to/from a UDP stream.
pub const HOPR_UDP_BUFFER_SIZE: usize = 16384;

/// Size of the queue (back-pressure) for data incoming from a UDP stream.
pub const HOPR_UDP_QUEUE_SIZE: usize = 8192;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_ACTIVE_CLIENTS: hopr_metrics::MultiGauge = hopr_metrics::MultiGauge::new(
        "hopr_session_hoprd_clients",
        "Number of clients connected at this Entry node",
        &["type"]
    ).unwrap();
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionTargetSpec {
    Plain(String),
    Sealed(#[serde_as(as = "serde_with::base64::Base64")] Vec<u8>),
    Service(ServiceId),
}

impl std::fmt::Display for SessionTargetSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionTargetSpec::Plain(t) => write!(f, "{t}"),
            SessionTargetSpec::Sealed(t) => write!(f, "$${}", base64::prelude::BASE64_URL_SAFE.encode(t)),
            SessionTargetSpec::Service(t) => write!(f, "#{t}"),
        }
    }
}

impl std::str::FromStr for SessionTargetSpec {
    type Err = HoprLibError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(stripped) = s.strip_prefix("$$") {
            Self::Sealed(
                base64::prelude::BASE64_URL_SAFE
                    .decode(stripped)
                    .map_err(|e| HoprLibError::GeneralError(e.to_string()))?,
            )
        } else if let Some(stripped) = s.strip_prefix("#") {
            Self::Service(
                stripped
                    .parse()
                    .map_err(|_| HoprLibError::GeneralError("cannot parse service id".into()))?,
            )
        } else {
            Self::Plain(s.to_owned())
        })
    }
}

impl SessionTargetSpec {
    pub fn into_target(self, protocol: IpProtocol) -> Result<SessionTarget, HoprLibError> {
        Ok(match (protocol, self) {
            (IpProtocol::TCP, SessionTargetSpec::Plain(plain)) => SessionTarget::TcpStream(
                IpOrHost::from_str(&plain)
                    .map(SealedHost::from)
                    .map_err(|e| HoprLibError::GeneralError(e.to_string()))?,
            ),
            (IpProtocol::UDP, SessionTargetSpec::Plain(plain)) => SessionTarget::UdpStream(
                IpOrHost::from_str(&plain)
                    .map(SealedHost::from)
                    .map_err(|e| HoprLibError::GeneralError(e.to_string()))?,
            ),
            (IpProtocol::TCP, SessionTargetSpec::Sealed(enc)) => {
                SessionTarget::TcpStream(SealedHost::Sealed(enc.into_boxed_slice()))
            }
            (IpProtocol::UDP, SessionTargetSpec::Sealed(enc)) => {
                SessionTarget::UdpStream(SealedHost::Sealed(enc.into_boxed_slice()))
            }
            (_, SessionTargetSpec::Service(id)) => SessionTarget::ExitNode(id),
        })
    }
}

/// Entry stored in the session registry table.
#[derive(Debug)]
pub struct StoredSessionEntry {
    /// Target of the Session.
    pub target: SessionTargetSpec,
    /// Routing used for the Session.
    pub path: RoutingOptions,
    /// The join handle for the Session processing.
    pub jh: hopr_async_runtime::prelude::JoinHandle<()>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionWebsocketClientQueryRequest {
    #[schema(required = true)]
    #[serde_as(as = "DisplayFromStr")]
    pub destination: PeerId,
    #[schema(required = true)]
    pub hops: u8,
    #[cfg(feature = "explicit-path")]
    #[schema(required = false)]
    pub path: Option<String>,
    #[schema(required = true)]
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub capabilities: Vec<SessionCapability>,
    #[schema(required = true)]
    #[serde_as(as = "DisplayFromStr")]
    pub target: SessionTargetSpec,
    #[schema(required = false)]
    #[serde(default = "default_protocol")]
    pub protocol: IpProtocol,
}

#[inline]
fn default_protocol() -> IpProtocol {
    IpProtocol::TCP
}

impl SessionWebsocketClientQueryRequest {
    pub(crate) fn into_protocol_session_config(self) -> Result<SessionClientConfig, HoprLibError> {
        #[cfg(not(feature = "explicit-path"))]
        let path_options = RoutingOptions::Hops((self.hops as u32).try_into()?);

        #[cfg(feature = "explicit-path")]
        let path_options = if let Some(path) = self.path {
            // Explicit `path` will override `hops`
            RoutingOptions::IntermediatePath(
                path.split(',')
                    .map(PeerId::from_str)
                    .collect::<Result<Vec<PeerId>, _>>()
                    .map_err(|e| HoprLibError::GeneralError(format!("invalid peer id on path: {e}")))?
                    .try_into()?,
            )
        } else {
            RoutingOptions::Hops((self.hops as u32).try_into()?)
        };

        Ok(SessionClientConfig {
            peer: self.destination,
            path_options,
            target: self.target.into_target(self.protocol)?,
            capabilities: self.capabilities,
        })
    }
}

#[derive(Debug, Default, Clone, Deserialize, utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
#[allow(dead_code)] // not dead code, just for codegen
struct WssData(Vec<u8>);

/// Websocket endpoint exposing a binary socket-like connection to a peer through websockets using underlying HOPR sessions.
///
/// Once configured, the session represents and automatically managed connection to a target peer through a network routing
/// configuration. The session can be used to send and receive binary data over the network.
///
/// Authentication (if enabled) is done by cookie `X-Auth-Token`.
///
/// Connect to the endpoint by using a WS client. No preview available. Example: `ws://127.0.0.1:3001/api/v3/session/websocket
#[allow(dead_code)] // not dead code, just for documentation
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/session/websocket"),
        params(SessionWebsocketClientQueryRequest),
        responses(
            (status = 200, description = "Successfully created a new client websocket session."),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError),
            (status = 429, description = "Too many open websocket connections.", body = ApiError),
        ),
        security(
            ("api_token" = []),
            ("bearer_token" = [])
        ),
        tag = "Session",
    )]

pub(crate) async fn websocket(
    ws: WebSocketUpgrade,
    Query(query): Query<SessionWebsocketClientQueryRequest>,
    State(state): State<Arc<InternalState>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let data = query.into_protocol_session_config().map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
    })?;

    let hopr = state.hopr.clone();
    let session: HoprSession = hopr.connect_to(data).await.map_err(|e| {
        error!(error = %e, "Failed to establish session");
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
    })?;

    Ok::<_, (StatusCode, ApiErrorStatus)>(ws.on_upgrade(move |socket| websocket_connection(socket, session)))
}

enum WebSocketInput {
    Network(Result<Box<[u8]>, std::io::Error>),
    WsInput(Result<Message, Error>),
}

/// The maximum number of bytes read from a Session that WS can transfer within a single message.
const WS_MAX_SESSION_READ_SIZE: usize = 4096;

#[tracing::instrument(level = "debug", skip(socket, session))]
async fn websocket_connection(socket: WebSocket, session: HoprSession) {
    let session_id = *session.id();

    let (rx, mut tx) = session.split();
    let (mut sender, receiver) = socket.split();

    let mut queue = (
        receiver.map(WebSocketInput::WsInput),
        AsyncReadStreamer::<WS_MAX_SESSION_READ_SIZE, _>(rx).map(WebSocketInput::Network),
    )
        .merge();

    let (mut bytes_to_session, mut bytes_from_session) = (0, 0);

    while let Some(v) = queue.next().await {
        match v {
            WebSocketInput::Network(bytes) => match bytes {
                Ok(bytes) => {
                    let len = bytes.len();
                    if let Err(e) = sender.send(Message::Binary(bytes.into())).await {
                        error!(
                            error = %e,
                            "Failed to emit read data onto the websocket, closing connection"
                        );
                        break;
                    };
                    bytes_from_session += len;
                }
                Err(e) => {
                    error!(
                        error = %e,
                        "Failed to push data from network to socket, closing connection"
                    );
                    break;
                }
            },
            WebSocketInput::WsInput(ws_in) => match ws_in {
                Ok(Message::Binary(data)) => {
                    let len = data.len();
                    if let Err(e) = tx.write(data.as_ref()).await {
                        error!(error = %e, "Failed to write data to the session, closing connection");
                        break;
                    }
                    bytes_to_session += len;
                }
                Ok(Message::Text(_)) => {
                    error!("Received string instead of binary data, closing connection");
                    break;
                }
                Ok(Message::Close(_)) => {
                    debug!("Received close frame, closing connection");
                    break;
                }
                Ok(m) => trace!(message = ?m, "skipping an unsupported websocket message"),
                Err(e) => {
                    error!(error = %e, "Failed to get a valid websocket message, closing connection");
                    break;
                }
            },
        }
    }

    info!(%session_id, bytes_from_session, bytes_to_session, "WS session connection ended");
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "destination": "12D3KooWR4uwjKCDCAY1xsEFB4esuWLF9Q5ijYvCjz5PNkTbnu33",
        "path": {
            "Hops": 1
        },
        "target": {"Plain": "localhost:8080"},
        "listenHost": "127.0.0.1:10000",
        "capabilities": ["Retransmission", "Segmentation"]
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientRequest {
    /// Peer ID of the Exit node.
    #[serde_as(as = "DisplayFromStr")]
    pub destination: PeerOrAddress,
    /// HOPR routing options for the Session.
    pub path: RoutingOptions,
    /// Session target specification.
    pub target: SessionTargetSpec,
    /// Listen host (`ip:port`) for the Session socket at the Entry node.
    ///
    /// Supports also partial specification (only `ip` or only `:port`) with the
    /// respective part replaced by the node's configured default.
    pub listen_host: Option<String>,
    #[serde_as(as = "Option<Vec<DisplayFromStr>>")]
    /// Capabilities for the Session protocol.
    ///
    /// Defaults to `Segmentation` and `Retransmission` for TCP and nothing for UDP.
    pub capabilities: Option<Vec<SessionCapability>>,
}

impl SessionClientRequest {
    pub(crate) fn into_protocol_session_config(
        self,
        target_protocol: IpProtocol,
    ) -> Result<SessionClientConfig, HoprLibError> {
        let peer = match self.destination {
            PeerOrAddress::PeerId(peer_id) => peer_id,
            PeerOrAddress::Address(address) => {
                return Err(HoprLibError::GeneralError(format!("invalid destination: {}", address)))
            }
        };

        #[cfg(not(feature = "explicit-path"))]
        if matches!(&self.path, RoutingOptions::IntermediatePath(_)) {
            return Err(HoprLibError::GeneralError("explicit paths are not allowed".into()));
        }

        Ok(SessionClientConfig {
            peer,
            path_options: self.path,
            target: self.target.into_target(target_protocol)?,
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
        "target": "example.com:80",
        "protocol": "tcp",
        "ip": "127.0.0.1",
        "port": 5542,
        "path": { "Hops": 1 }
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionClientResponse {
    pub target: String,
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    pub protocol: IpProtocol,
    pub ip: String,
    pub path: RoutingOptions,
    pub port: u16,
}

/// This function first tries to parse `requested` as the `ip:port` host pair.
/// If that does not work, it tries to parse `requested` as a single IP address
/// and as a `:` prefixed port number. Whichever of those fails, is replaced by the corresponding
/// part from the given `default`.
fn build_binding_host(requested: Option<&str>, default: std::net::SocketAddr) -> std::net::SocketAddr {
    match requested.map(|r| std::net::SocketAddr::from_str(r).map_err(|_| r)) {
        Some(Err(requested)) => {
            // If the requested host is not parseable as a whole as `SocketAddr`, try only its parts
            debug!(requested, %default, "using partially default listen host");
            std::net::SocketAddr::new(
                requested.parse().unwrap_or(default.ip()),
                requested
                    .strip_prefix(":")
                    .and_then(|p| u16::from_str(p).ok())
                    .unwrap_or(default.port()),
            )
        }
        Some(Ok(requested)) => {
            debug!(%requested, "using requested listen host");
            requested
        }
        None => {
            debug!(%default, "using default listen host");
            default
        }
    }
}

/// Creates a new client session returning the given session listening host and port over TCP or UDP.
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
        params(
            ("protocol" = String, Path, description = "IP transport protocol")
        ),
        request_body(
            content = SessionClientRequest,
            description = "Creates a new client HOPR session that will start listening on a dedicated port. Once the port is bound, it is possible to use the socket for bidirectional read and write communication.",
            content_type = "application/json"),
        responses(
            (status = 200, description = "Successfully created a new client session.", body = SessionClientResponse),
            (status = 400, description = "Invalid IP protocol.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 409, description = "Listening address and port already in use.", body = ApiError),
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
    let bind_host: std::net::SocketAddr = build_binding_host(args.listen_host.as_deref(), state.default_listen_host);

    if bind_host.port() > 0
        && state
            .open_listeners
            .read()
            .await
            .contains_key(&ListenerId(protocol, bind_host))
    {
        return Err((StatusCode::CONFLICT, ApiErrorStatus::ListenHostAlreadyUsed));
    }

    let target = args.target.clone();
    let path = args.path.clone();
    let data = args.into_protocol_session_config(protocol).map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
    })?;

    // TODO: consider pooling the sessions on a listener, so that the negotiation is amortized

    debug!("binding {protocol} session listening socket to {bind_host}");
    let bound_host = match protocol {
        IpProtocol::TCP => {
            // Bind the TCP socket first
            let (bound_host, tcp_listener) = tcp_listen_on(bind_host).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    (StatusCode::CONFLICT, ApiErrorStatus::ListenHostAlreadyUsed)
                } else {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ApiErrorStatus::UnknownFailure(format!("failed to start TCP listener on {bind_host}: {e}")),
                    )
                }
            })?;
            info!(%bound_host, "TCP session listener bound");

            // For each new TCP connection coming to the listener,
            // open a Session with the same parameters
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
                                    debug!(socket = ?sock_addr, "incoming TCP connection");
                                    let session = match hopr.connect_to(data).await {
                                        Ok(s) => s,
                                        Err(e) => {
                                            error!(error = %e, "failed to establish session");
                                            return;
                                        }
                                    };

                                    debug!(
                                        socket = ?sock_addr,
                                        session_id = tracing::field::debug(*session.id()),
                                        "new session for incoming TCP connection",
                                    );

                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    METRIC_ACTIVE_CLIENTS.increment(&["tcp"], 1.0);

                                    bind_session_to_stream(session, stream, HOPR_TCP_BUFFER_SIZE).await;

                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    METRIC_ACTIVE_CLIENTS.decrement(&["tcp"], 1.0);
                                }
                                Err(e) => error!(error = %e, "failed to accept connection"),
                            }
                        }
                    }),
            );

            state.open_listeners.write().await.insert(
                ListenerId(protocol, bound_host),
                StoredSessionEntry {
                    target: target.clone(),
                    path: path.clone(),
                    jh,
                },
            );
            bound_host
        }
        IpProtocol::UDP => {
            // Bind the UDP socket first
            let (bound_host, udp_socket) = udp_bind_to(bind_host).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::AddrInUse {
                    (StatusCode::CONFLICT, ApiErrorStatus::ListenHostAlreadyUsed)
                } else {
                    (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        ApiErrorStatus::UnknownFailure(format!("failed to start UDP listener on {bind_host}: {e}")),
                    )
                }
            })?;

            info!(%bound_host, "UDP session listener bound");

            let hopr = state.hopr.clone();

            // Create a single session for the UDP socket
            let session = hopr.connect_to(data).await.map_err(|e| {
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(e.to_string()),
                )
            })?;

            let open_listeners_clone = state.open_listeners.clone();
            let listener_id = ListenerId(protocol, bound_host);

            state.open_listeners.write().await.insert(
                listener_id,
                StoredSessionEntry {
                    target: target.clone(),
                    path: path.clone(),
                    jh: hopr_async_runtime::prelude::spawn(async move {
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_ACTIVE_CLIENTS.increment(&["udp"], 1.0);

                        bind_session_to_stream(session, udp_socket, HOPR_UDP_BUFFER_SIZE).await;

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_ACTIVE_CLIENTS.decrement(&["udp"], 1.0);

                        // Once the Session closes, remove it from the list
                        open_listeners_clone.write().await.remove(&listener_id);
                    }),
                },
            );
            bound_host
        }
    };

    Ok::<_, (StatusCode, ApiErrorStatus)>(
        (
            StatusCode::OK,
            Json(SessionClientResponse {
                protocol,
                path,
                target: target.to_string(),
                ip: bound_host.ip().to_string(),
                port: bound_host.port(),
            }),
        )
            .into_response(),
    )
}

/// Lists existing Session listeners for the given IP protocol.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/session/{{protocol}}"),
    params(
            ("protocol" = String, Path, description = "IP transport protocol")
    ),
    responses(
            (status = 200, description = "Opened session listeners for the given IP protocol.", body = Vec<SessionClientResponse>),
            (status = 400, description = "Invalid IP protocol.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError)
    ),
    security(
            ("api_token" = []),
            ("bearer_token" = [])
    ),
    tag = "Session",
)]
pub(crate) async fn list_clients(
    State(state): State<Arc<InternalState>>,
    Path(protocol): Path<IpProtocol>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let response = state
        .open_listeners
        .read()
        .await
        .iter()
        .filter(|(id, _)| id.0 == protocol)
        .map(|(id, entry)| SessionClientResponse {
            protocol,
            target: entry.target.to_string(),
            ip: id.1.ip().to_string(),
            port: id.1.port(),
            path: entry.path.clone(),
        })
        .collect::<Vec<_>>();

    Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::OK, Json(response)).into_response())
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
/// If the passed port number is 0, listeners on all ports of the given listening IP and protocol
/// will be closed.
#[utoipa::path(
    delete,
    path = const_format::formatcp!("{BASE_PATH}/session/{{protocol}}"),
    params(
            ("protocol" = String, Path, description = "IP transport protocol")
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
    let listening_ip: IpAddr = listening_ip
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidInput))?;

    {
        let mut open_listeners = state.open_listeners.write().await;

        let mut to_remove = Vec::new();

        // Find all listeners with protocol, listening IP and optionally port number (if > 0)
        open_listeners
            .iter()
            .filter(|(ListenerId(proto, addr), _)| {
                protocol == *proto && addr.ip() == listening_ip && (addr.port() == port || port == 0)
            })
            .for_each(|(id, _)| to_remove.push(*id));

        if to_remove.is_empty() {
            return Err((StatusCode::NOT_FOUND, ApiErrorStatus::InvalidInput));
        }

        for bound_addr in to_remove {
            let entry = open_listeners
                .remove(&bound_addr)
                .ok_or((StatusCode::NOT_FOUND, ApiErrorStatus::InvalidInput))?;

            hopr_async_runtime::prelude::cancel_join_handle(entry.jh).await;
        }
    }

    Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::NO_CONTENT, "").into_response())
}

async fn try_restricted_bind<'a, F, S, Fut>(
    addrs: Vec<std::net::SocketAddr>,
    range_str: &str,
    binder: F,
) -> std::io::Result<S>
where
    F: Fn(Vec<std::net::SocketAddr>) -> Fut,
    Fut: Future<Output = std::io::Result<S>>,
{
    if addrs.is_empty() {
        return Err(std::io::Error::other("no valid socket addresses found"));
    }

    let range = range_str
        .split_once(":")
        .and_then(
            |(a, b)| match u16::from_str(a).and_then(|a| Ok((a, u16::from_str(b)?))) {
                Ok((a, b)) if a <= b => Some(a..=b),
                _ => None,
            },
        )
        .ok_or(std::io::Error::other(format!("invalid port range {range_str}")))?;

    for port in range {
        let addrs = addrs
            .iter()
            .map(|addr| std::net::SocketAddr::new(addr.ip(), port))
            .collect::<Vec<_>>();
        match binder(addrs).await {
            Ok(listener) => return Ok(listener),
            Err(error) => debug!(%error, "listen address not usable"),
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AddrNotAvailable,
        format!("no valid socket addresses found within range: {range_str}"),
    ))
}

async fn tcp_listen_on<A: std::net::ToSocketAddrs>(address: A) -> std::io::Result<(std::net::SocketAddr, TcpListener)> {
    let addrs = address.to_socket_addrs()?.collect::<Vec<_>>();

    // If automatic port allocation is requested and there's a restriction on the port range
    // (via HOPRD_SESSION_PORT_RANGE), try to find an address within that range.
    if addrs.iter().all(|a| a.port() == 0) {
        if let Ok(range_str) = std::env::var(crate::env::HOPRD_SESSION_PORT_RANGE) {
            let tcp_listener =
                try_restricted_bind(
                    addrs,
                    &range_str,
                    |a| async move { TcpListener::bind(a.as_slice()).await },
                )
                .await?;
            return Ok((tcp_listener.local_addr()?, tcp_listener));
        }
    }

    let tcp_listener = TcpListener::bind(addrs.as_slice()).await?;
    Ok((tcp_listener.local_addr()?, tcp_listener))
}

async fn udp_bind_to<A: std::net::ToSocketAddrs>(
    address: A,
) -> std::io::Result<(std::net::SocketAddr, ConnectedUdpStream)> {
    let addrs = address.to_socket_addrs()?.collect::<Vec<_>>();

    let builder = ConnectedUdpStream::builder()
        .with_buffer_size(HOPR_UDP_BUFFER_SIZE)
        .with_foreign_data_mode(ForeignDataMode::Discard) // discard data from UDP clients other than the first one served
        .with_queue_size(HOPR_UDP_QUEUE_SIZE)
        .with_receiver_parallelism(UdpStreamParallelism::Auto);

    // If automatic port allocation is requested and there's a restriction on the port range
    // (via HOPRD_SESSION_PORT_RANGE), try to find an address within that range.
    if addrs.iter().all(|a| a.port() == 0) {
        if let Ok(range_str) = std::env::var(crate::env::HOPRD_SESSION_PORT_RANGE) {
            let udp_listener = try_restricted_bind(addrs, &range_str, |addrs| {
                futures::future::ready(builder.clone().build(addrs.as_slice()))
            })
            .await?;

            return Ok((*udp_listener.bound_address(), udp_listener));
        }
    }

    let udp_socket = builder.build(address)?;
    Ok((*udp_socket.bound_address(), udp_socket))
}

async fn bind_session_to_stream<T>(mut session: HoprSession, mut stream: T, max_buf: usize)
where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let session_id = *session.id();
    match transfer_session(&mut session, &mut stream, max_buf).await {
        Ok((session_to_stream_bytes, stream_to_session_bytes)) => info!(
            session_id = ?session_id,
            session_to_stream_bytes, stream_to_session_bytes, "client session ended",
        ),
        Err(error) => error!(
            session_id = ?session_id,
            %error,
            "error during data transfer"
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
                .map_err(|_| TransportSessionError::Closed)?;

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

        let (bound_addr, tcp_listener) = tcp_listen_on(("127.0.0.1", 0)).await.context("listen_on failed")?;

        tokio::task::spawn(async move {
            match tcp_listener.accept().await {
                Ok((stream, _)) => bind_session_to_stream(session, stream, HOPR_TCP_BUFFER_SIZE).await,
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

        tokio::task::spawn(bind_session_to_stream(
            session,
            udp_listener,
            hopr_lib::SESSION_USABLE_MTU_SIZE,
        ));

        let mut udp_stream = ConnectedUdpStream::builder()
            .with_buffer_size(hopr_lib::SESSION_USABLE_MTU_SIZE)
            .with_queue_size(HOPR_UDP_QUEUE_SIZE)
            .with_counterparty(listen_addr)
            .build(("127.0.0.1", 0))
            .context("bind failed")?;

        let data = vec![b"hello", b"world", b"this ", b"is   ", b"    a", b" test"];

        for d in data.clone().into_iter() {
            udp_stream.write_all(d).await.context("write failed")?;
        }

        for d in data.iter() {
            let mut buf = vec![0; d.len()];
            udp_stream.read_exact(&mut buf).await.context("read failed")?;
        }

        Ok(())
    }

    #[test]
    fn test_build_binding_address() {
        let default = "10.0.0.1:10000".parse().unwrap();

        let result = build_binding_host(Some("127.0.0.1:10000"), default);
        assert_eq!(result, "127.0.0.1:10000".parse::<std::net::SocketAddr>().unwrap());

        let result = build_binding_host(None, default);
        assert_eq!(result, "10.0.0.1:10000".parse::<std::net::SocketAddr>().unwrap());

        let result = build_binding_host(Some("127.0.0.1"), default);
        assert_eq!(result, "127.0.0.1:10000".parse::<std::net::SocketAddr>().unwrap());

        let result = build_binding_host(Some(":1234"), default);
        assert_eq!(result, "10.0.0.1:1234".parse::<std::net::SocketAddr>().unwrap());

        let result = build_binding_host(Some(":"), default);
        assert_eq!(result, "10.0.0.1:10000".parse::<std::net::SocketAddr>().unwrap());

        let result = build_binding_host(Some(""), default);
        assert_eq!(result, "10.0.0.1:10000".parse::<std::net::SocketAddr>().unwrap());
    }
}
