use std::{
    collections::VecDeque,
    fmt::Formatter,
    future::Future,
    hash::Hash,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    Error,
    extract::{
        Json, Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::status::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::Query;
use base64::Engine;
use dashmap::DashMap;
use futures::{
    AsyncReadExt, AsyncWriteExt, FutureExt, SinkExt, StreamExt, TryStreamExt,
    future::{AbortHandle, AbortRegistration},
};
use futures_concurrency::stream::Merge;
use hopr_lib::{
    Address, Hopr, HoprSession, HoprSessionId, HoprTransportError, SESSION_MTU, SURB_SIZE, ServiceId,
    SessionCapabilities, SessionClientConfig, SessionManagerError, SessionTarget, SurbBalancerConfig,
    TransportSessionError, errors::HoprLibError, transfer_session,
};
use hopr_network_types::{
    prelude::{ConnectedUdpStream, IpOrHost, SealedHost, UdpStreamParallelism},
    udp::ForeignDataMode,
    utils::AsyncReadStreamer,
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tokio::net::TcpListener;
use tracing::{debug, error, info, trace};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState, ListenerId};

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

// Imported for some IDEs to not treat the `json!` macro inside the `schema` macro as an error
#[allow(unused_imports)]
use serde_json::json;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[schema(
    example = json!({"Plain": "example.com:80"}),
    example = json!({"Sealed": "SGVsbG9Xb3JsZA"}), // base64 for "HelloWorld"
    example = json!({"Service": 0})
)]
/// Session target specification.
pub enum SessionTargetSpec {
    Plain(String),
    Sealed(#[serde_as(as = "serde_with::base64::Base64")] Vec<u8>),
    #[schema(value_type = u32)]
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

impl FromStr for SessionTargetSpec {
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
    /// Destination address of the Session counterparty.
    pub destination: Address,
    /// Target of the Session.
    pub target: SessionTargetSpec,
    /// Forward path used for the Session.
    pub forward_path: RoutingOptions,
    /// Return path used for the Session.
    pub return_path: RoutingOptions,
    /// The abort handle for the Session processing.
    pub abort_handle: AbortHandle,

    clients: Arc<DashMap<HoprSessionId, (SocketAddr, AbortHandle)>>,
}

#[repr(u8)]
#[derive(
    Debug,
    Clone,
    strum::EnumIter,
    strum::Display,
    strum::EnumString,
    Serialize,
    Deserialize,
    PartialEq,
    utoipa::ToSchema,
)]
#[schema(example = "Segmentation")]
/// Session capabilities that can be negotiated with the target peer.
pub enum SessionCapability {
    /// Frame segmentation
    Segmentation,
    /// Frame retransmission (ACK and NACK-based)
    Retransmission,
    /// Frame retransmission (only ACK-based)
    RetransmissionAckOnly,
    /// Disable packet buffering
    NoDelay,
    /// Disable SURB-based egress rate control at the Exit.
    NoRateControl,
}

impl From<SessionCapability> for hopr_lib::SessionCapabilities {
    fn from(cap: SessionCapability) -> hopr_lib::SessionCapabilities {
        match cap {
            SessionCapability::Segmentation => hopr_lib::SessionCapability::Segmentation.into(),
            SessionCapability::Retransmission => {
                hopr_lib::SessionCapability::RetransmissionNack | hopr_lib::SessionCapability::RetransmissionAck
            }
            SessionCapability::RetransmissionAckOnly => hopr_lib::SessionCapability::RetransmissionAck.into(),
            SessionCapability::NoDelay => hopr_lib::SessionCapability::NoDelay.into(),
            SessionCapability::NoRateControl => hopr_lib::SessionCapability::NoRateControl.into(),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionWebsocketClientQueryRequest {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(required = true, value_type = String)]
    pub destination: Address,
    #[schema(required = true)]
    pub hops: u8,
    #[cfg(feature = "explicit-path")]
    #[schema(required = false, value_type = String)]
    pub path: Option<Vec<Address>>,
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
    pub(crate) async fn into_protocol_session_config(
        self,
    ) -> Result<(Address, SessionTarget, SessionClientConfig), ApiErrorStatus> {
        #[cfg(not(feature = "explicit-path"))]
        let path_options = hopr_lib::RoutingOptions::Hops((self.hops as u32).try_into()?);

        #[cfg(feature = "explicit-path")]
        let path_options = if let Some(path) = self.path {
            // Explicit `path` will override `hops`
            hopr_lib::RoutingOptions::IntermediatePath(path.try_into()?)
        } else {
            hopr_lib::RoutingOptions::Hops((self.hops as u32).try_into()?)
        };

        let mut capabilities = SessionCapabilities::empty();
        capabilities.extend(self.capabilities.into_iter().flat_map(SessionCapabilities::from));

        Ok((
            self.destination,
            self.target.into_target(self.protocol)?,
            SessionClientConfig {
                forward_path_options: path_options.clone(),
                return_path_options: path_options.clone(), // TODO: allow using separate return options
                capabilities,
                ..Default::default()
            },
        ))
    }
}

#[derive(Debug, Default, Clone, Deserialize, utoipa::ToSchema)]
#[schema(value_type = String, format = Binary)]
#[allow(dead_code)] // not dead code, just for codegen
struct WssData(Vec<u8>);

/// Websocket endpoint exposing a binary socket-like connection to a peer through websockets using underlying HOPR
/// sessions.
///
/// Once configured, the session represents and automatically managed connection to a target peer through a network
/// routing configuration. The session can be used to send and receive binary data over the network.
///
/// Authentication (if enabled) is done by cookie `X-Auth-Token`.
///
/// Connect to the endpoint by using a WS client. No preview is available. Example:
/// `ws://127.0.0.1:3001/api/v4/session/websocket`
#[allow(dead_code)] // not dead code, just for documentation
#[utoipa::path(
        get,
        path = const_format::formatcp!("{BASE_PATH}/session/websocket"),
        description = "Websocket endpoint exposing a binary socket-like connection to a peer through websockets using underlying HOPR sessions.",
        request_body(
            content = SessionWebsocketClientQueryRequest,
            content_type = "application/json",
            description = "Websocket endpoint exposing a binary socket-like connection to a peer through websockets using underlying HOPR sessions.",
        ),
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
    let (dst, target, data) = query
        .into_protocol_session_config()
        .await
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;

    let hopr = state.hopr.clone();
    let session: HoprSession = hopr.connect_to(dst, target, data).await.map_err(|e| {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "Hops": 1 }))]
/// Routing options for the Session.
pub enum RoutingOptions {
    #[cfg(feature = "explicit-path")]
    #[schema(value_type = Vec<String>)]
    IntermediatePath(#[serde_as(as = "Vec<DisplayFromStr>")] Vec<Address>),
    Hops(usize),
}

impl RoutingOptions {
    pub(crate) async fn resolve(self) -> Result<hopr_lib::RoutingOptions, ApiErrorStatus> {
        Ok(match self {
            #[cfg(feature = "explicit-path")]
            RoutingOptions::IntermediatePath(path) => hopr_lib::RoutingOptions::IntermediatePath(path.try_into()?),
            RoutingOptions::Hops(hops) => hopr_lib::RoutingOptions::Hops(hops.try_into()?),
        })
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "destination": "0x1B482420Afa04aeC1Ef0e4a00C18451E84466c75",
        "forwardPath": { "Hops": 1 },
        "returnPath": { "Hops": 1 },
        "target": {"Plain": "localhost:8080"},
        "listenHost": "127.0.0.1:10000",
        "capabilities": ["Retransmission", "Segmentation"],
        "responseBuffer": "2 MB",
        "maxSurbUpstream": "2000 kb/s",
        "sessionPool": 0,
        "maxClientSessions": 2
    }))]
#[serde(rename_all = "camelCase")]
/// Request body for creating a new client session.
pub(crate) struct SessionClientRequest {
    /// Address of the Exit node.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    pub destination: Address,
    /// The forward path for the Session.
    pub forward_path: RoutingOptions,
    /// The return path for the Session.
    pub return_path: RoutingOptions,
    /// Target for the Session.
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
    /// The amount of response data the Session counterparty can deliver back to us,
    /// without us sending any SURBs to them.
    ///
    /// In other words, this size is recalculated to a number of SURBs delivered
    /// to the counterparty upfront and then maintained.
    /// The maintenance is dynamic, based on the number of responses we receive.
    ///
    /// All syntaxes like "2 MB", "128 kiB", "3MiB" are supported. The value must be
    /// at least the size of 2 Session packet payloads.
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = String)]
    pub response_buffer: Option<bytesize::ByteSize>,
    /// The maximum throughput at which artificial SURBs might be generated and sent
    /// to the recipient of the Session.
    ///
    /// On Sessions that rarely send data but receive a lot (= Exit node has high SURB consumption),
    /// this should roughly match the maximum retrieval throughput.
    ///
    /// All syntaxes like "2 MBps", "1.2Mbps", "300 kb/s", "1.23 Mb/s" are supported.
    #[serde(default)]
    #[serde(with = "human_bandwidth::option")]
    #[schema(value_type = String)]
    pub max_surb_upstream: Option<human_bandwidth::re::bandwidth::Bandwidth>,
    /// How many Sessions to pool for clients.
    ///
    /// If no sessions are pooled, they will be opened ad-hoc when a client connects.
    /// It has no effect on UDP sessions in the current implementation.
    ///
    /// Currently, the maximum value is 5.
    pub session_pool: Option<usize>,
    /// The maximum number of client sessions that the listener can spawn.
    ///
    /// This currently applies only to the TCP sessions, as UDP sessions cannot
    /// handle multiple clients (and spawn therefore always only a single session).
    ///
    /// If this value is smaller than the value specified in `session_pool`, it will
    /// be set to that value.
    ///
    /// The default value is 5.
    pub max_client_sessions: Option<usize>,
}

impl SessionClientRequest {
    pub(crate) async fn into_protocol_session_config(
        self,
        target_protocol: IpProtocol,
    ) -> Result<(Address, SessionTarget, SessionClientConfig), ApiErrorStatus> {
        Ok((
            self.destination,
            self.target.into_target(target_protocol)?,
            SessionClientConfig {
                forward_path_options: self.forward_path.resolve().await?,
                return_path_options: self.return_path.resolve().await?,
                capabilities: self
                    .capabilities
                    .map(|vs| {
                        let mut caps = SessionCapabilities::empty();
                        caps.extend(vs.into_iter().map(SessionCapabilities::from));
                        caps
                    })
                    .unwrap_or_else(|| match target_protocol {
                        IpProtocol::TCP => {
                            hopr_lib::SessionCapability::RetransmissionAck
                                | hopr_lib::SessionCapability::RetransmissionNack
                                | hopr_lib::SessionCapability::Segmentation
                        }
                        // Only Segmentation capability for UDP per default
                        _ => SessionCapability::Segmentation.into(),
                    }),
                surb_management: SessionConfig {
                    response_buffer: self.response_buffer,
                    max_surb_upstream: self.max_surb_upstream,
                }
                .into(),
                ..Default::default()
            },
        ))
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "destination": "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F",
        "target": "example.com:80",
        "forwardPath": { "Hops": 1 },
        "returnPath": { "Hops": 1 },
        "protocol": "tcp",
        "ip": "127.0.0.1",
        "port": 5542,
        "mtu": 1020,
        "surb_len": 400,
        "active_clients": []
    }))]
#[serde(rename_all = "camelCase")]
/// Response body for creating a new client session.
pub(crate) struct SessionClientResponse {
    #[schema(example = "example.com:80")]
    /// Target of the Session.
    pub target: String,
    /// Destination node (exit node) of the Session.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String)]
    pub destination: Address,
    /// Forward routing path.
    pub forward_path: RoutingOptions,
    /// Return routing path.
    pub return_path: RoutingOptions,
    /// IP protocol used by Session's listening socket.
    #[serde_as(as = "DisplayFromStr")]
    #[schema(example = "tcp")]
    pub protocol: IpProtocol,
    /// Listening IP address of the Session's socket.
    #[schema(example = "127.0.0.1")]
    pub ip: String,
    #[schema(example = 5542)]
    /// Listening port of the Session's socket.
    pub port: u16,
    /// MTU used by the Session.
    pub mtu: usize,
    /// Size of a Single Use Reply Block used by the protocol.
    ///
    /// This is usefult for SURB balancing calculations.
    pub surb_len: usize,
    /// Lists Session IDs of all active clients.
    ///
    /// Can contain multiple entries on TCP sessions, but currently
    /// always only a single entry on UDP sessions.
    pub active_clients: Vec<String>,
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

struct SessionPool {
    pool: Option<Arc<std::sync::Mutex<VecDeque<HoprSession>>>>,
    ah: Option<AbortHandle>,
}

impl SessionPool {
    pub const MAX_SESSION_POOL_SIZE: usize = 5;

    async fn new(
        size: usize,
        dst: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
        hopr: Arc<Hopr>,
    ) -> Result<Self, (StatusCode, ApiErrorStatus)> {
        let pool = Arc::new(std::sync::Mutex::new(VecDeque::with_capacity(size)));
        let hopr_clone = hopr.clone();
        let pool_clone = pool.clone();
        futures::stream::iter(0..size.min(Self::MAX_SESSION_POOL_SIZE))
            .map(Ok)
            .try_for_each_concurrent(Self::MAX_SESSION_POOL_SIZE, move |i| {
                let pool = pool_clone.clone();
                let hopr = hopr_clone.clone();
                let target = target.clone();
                let cfg = cfg.clone();
                async move {
                    match hopr.connect_to(dst, target.clone(), cfg.clone()).await {
                        Ok(s) => {
                            debug!(session_id = %s.id(), num_session = i, "created a new session in pool");
                            pool.lock()
                                .map_err(|_| {
                                    (
                                        StatusCode::INTERNAL_SERVER_ERROR,
                                        ApiErrorStatus::UnknownFailure("lock failed".into()),
                                    )
                                })?
                                .push_back(s);
                            Ok(())
                        }
                        Err(error) => {
                            error!(%error, num_session = i, "failed to establish session for pool");
                            Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                ApiErrorStatus::UnknownFailure(format!(
                                    "failed to establish session #{i} in pool to {dst}: {error}"
                                )),
                            ))
                        }
                    }
                }
            })
            .await?;

        // Spawn a task that periodically sends keep alive messages to the Session in the pool.
        if !pool.lock().map(|p| p.is_empty()).unwrap_or(true) {
            let pool_clone_1 = pool.clone();
            let pool_clone_2 = pool.clone();
            let pool_clone_3 = pool.clone();
            Ok(Self {
                pool: Some(pool),
                ah: Some(hopr_async_runtime::spawn_as_abortable!(
                    futures_time::stream::interval(futures_time::time::Duration::from(
                        std::time::Duration::from_secs(1).max(hopr.config().session.idle_timeout / 2)
                    ))
                    .take_while(move |_| {
                        // Continue the infinite interval stream until there are sessions in the pool
                        futures::future::ready(pool_clone_1.lock().is_ok_and(|p| !p.is_empty()))
                    })
                    .flat_map(move |_| {
                        // Get all SessionIds of the remaining Sessions in the pool
                        let ids = pool_clone_2.lock().ok().map(|v| v.iter().map(|s| *s.id()).collect::<Vec<_>>());
                        futures::stream::iter(ids.into_iter().flatten())
                    })
                    .for_each(move |id| {
                        let hopr = hopr.clone();
                        let pool = pool_clone_3.clone();
                        async move {
                            // Make sure the Session is still alive, otherwise remove it from the pool
                            if let Err(error) = hopr.keep_alive_session(&id).await {
                                error!(%error, %dst, session_id = %id, "session in pool is not alive, removing from pool");
                                if let Ok(mut pool) = pool.lock() {
                                    pool.retain(|s| *s.id() != id);
                                }
                            }
                        }
                    })
                ))
            })
        } else {
            Ok(Self { pool: None, ah: None })
        }
    }

    fn pop(&mut self) -> Option<HoprSession> {
        self.pool.as_ref().and_then(|pool| pool.lock().ok()?.pop_front())
    }
}

impl Drop for SessionPool {
    fn drop(&mut self) {
        if let Some(ah) = self.ah.take() {
            ah.abort();
        }
    }
}

async fn create_tcp_client_binding(
    bind_host: std::net::SocketAddr,
    state: Arc<InternalState>,
    args: SessionClientRequest,
) -> Result<(std::net::SocketAddr, Option<HoprSessionId>), (StatusCode, ApiErrorStatus)> {
    let target_spec = args.target.clone();
    let (dst, target, data) = args
        .clone()
        .into_protocol_session_config(IpProtocol::TCP)
        .await
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;

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

    // Create a session pool if requested
    let session_pool_size = args.session_pool.unwrap_or(0);
    let mut session_pool = SessionPool::new(session_pool_size, dst, target.clone(), data.clone(), hopr.clone()).await?;

    let active_sessions = Arc::new(DashMap::new());
    let mut max_clients = args.max_client_sessions.unwrap_or(5).max(1);

    if max_clients < session_pool_size {
        max_clients = session_pool_size;
    }

    // Create an abort handler for the listener
    let (abort_handle, abort_reg) = AbortHandle::new_pair();
    let active_sessions_clone = active_sessions.clone();
    hopr_async_runtime::prelude::spawn(async move {
        let active_sessions_clone_2 = active_sessions_clone.clone();

        futures::stream::Abortable::new(tokio_stream::wrappers::TcpListenerStream::new(tcp_listener), abort_reg)
            .and_then(|sock| async { Ok((sock.peer_addr()?, sock)) })
            .for_each(move |accepted_client| {
                let data = data.clone();
                let target = target.clone();
                let hopr = hopr.clone();
                let active_sessions = active_sessions_clone_2.clone();

                // Try to pop from the pool only if a client was accepted
                let maybe_pooled_session = accepted_client.is_ok().then(|| session_pool.pop()).flatten();
                async move {
                    match accepted_client {
                        Ok((sock_addr, mut stream)) => {
                            debug!(?sock_addr, "incoming TCP connection");

                            // Check that we are still within the quota,
                            // otherwise shutdown the new client immediately
                            if active_sessions.len() >= max_clients {
                                error!(?bind_host, "no more client slots available at listener");
                                use tokio::io::AsyncWriteExt;
                                if let Err(error) = stream.shutdown().await {
                                    error!(%error, ?sock_addr, "failed to shutdown TCP connection");
                                }
                                return;
                            }

                            // See if we still have some session pooled
                            let session = match maybe_pooled_session {
                                Some(s) => {
                                    debug!(session_id = %s.id(), "using pooled session");
                                    s
                                }
                                None => {
                                    debug!("no more active sessions in the pool, creating a new one");
                                    match hopr.connect_to(dst, target, data).await {
                                        Ok(s) => s,
                                        Err(error) => {
                                            error!(%error, "failed to establish session");
                                            return;
                                        }
                                    }
                                }
                            };

                            let session_id = *session.id();
                            debug!(?sock_addr, %session_id, "new session for incoming TCP connection");

                            let (abort_handle, abort_reg) = AbortHandle::new_pair();
                            active_sessions.insert(session_id, (sock_addr, abort_handle));

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_ACTIVE_CLIENTS.increment(&["tcp"], 1.0);

                            hopr_async_runtime::prelude::spawn(
                                // The stream either terminates naturally (by the client closing the TCP connection)
                                // or is terminated via the abort handle.
                                bind_session_to_stream(session, stream, HOPR_TCP_BUFFER_SIZE, Some(abort_reg)).then(
                                    move |_| async move {
                                        // Regardless how the session ended, remove the abort handle
                                        // from the map
                                        active_sessions.remove(&session_id);

                                        debug!(%session_id, "tcp session has ended");

                                        #[cfg(all(feature = "prometheus", not(test)))]
                                        METRIC_ACTIVE_CLIENTS.decrement(&["tcp"], 1.0);
                                    },
                                ),
                            );
                        }
                        Err(error) => error!(%error, "failed to accept connection"),
                    }
                }
            })
            .await;

        // Once the listener is done, abort all active sessions created by the listener
        active_sessions_clone.iter().for_each(|entry| {
            let (sock_addr, handle) = entry.value();
            debug!(session_id = %entry.key(), ?sock_addr, "aborting opened TCP session after listener has been closed");
            handle.abort()
        });
    });

    state.open_listeners.write_arc().await.insert(
        ListenerId(hopr_network_types::types::IpProtocol::TCP, bound_host),
        StoredSessionEntry {
            destination: dst,
            target: target_spec.clone(),
            forward_path: args.forward_path.clone(),
            return_path: args.return_path.clone(),
            clients: active_sessions,
            abort_handle,
        },
    );
    Ok((bound_host, None))
}

async fn create_udp_client_binding(
    bind_host: std::net::SocketAddr,
    state: Arc<InternalState>,
    args: SessionClientRequest,
) -> Result<(std::net::SocketAddr, Option<HoprSessionId>), (StatusCode, ApiErrorStatus)> {
    let target_spec = args.target.clone();
    let (dst, target, data) = args
        .clone()
        .into_protocol_session_config(IpProtocol::UDP)
        .await
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;

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
    let session = hopr.connect_to(dst, target, data).await.map_err(|e| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )
    })?;

    let open_listeners_clone = state.open_listeners.clone();
    let listener_id = ListenerId(hopr_network_types::types::IpProtocol::UDP, bound_host);

    // Create an abort handle so that the Session can be terminated by aborting
    // the UDP stream first. Because under the hood, the bind_session_to_stream uses
    // `transfer_session` which in turn uses `copy_duplex_abortable`, aborting the
    // `udp_socket` will:
    //
    // 1. Initiate graceful shutdown of `udp_socket`
    // 2. Once done, initiate a graceful shutdown of `session`
    // 3. Finally, return from the `bind_session_to_stream` which will terminate the spawned task
    //
    // This is needed because the `udp_socket` cannot terminate by itself.
    let (abort_handle, abort_reg) = AbortHandle::new_pair();
    let clients = Arc::new(DashMap::new());
    // TODO: add multiple client support to UDP sessions (#7370)
    let session_id = *session.id();
    clients.insert(session_id, (bind_host, abort_handle.clone()));
    hopr_async_runtime::prelude::spawn(async move {
        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_ACTIVE_CLIENTS.increment(&["udp"], 1.0);

        bind_session_to_stream(session, udp_socket, HOPR_UDP_BUFFER_SIZE, Some(abort_reg)).await;

        #[cfg(all(feature = "prometheus", not(test)))]
        METRIC_ACTIVE_CLIENTS.decrement(&["udp"], 1.0);

        // Once the Session closes, remove it from the list
        open_listeners_clone.write_arc().await.remove(&listener_id);
    });

    state.open_listeners.write_arc().await.insert(
        listener_id,
        StoredSessionEntry {
            destination: dst,
            target: target_spec.clone(),
            forward_path: args.forward_path.clone(),
            return_path: args.return_path.clone(),
            abort_handle,
            clients,
        },
    );
    Ok((bound_host, Some(session_id)))
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
        description = "Creates a new client HOPR session that will start listening on a dedicated port. Once the port is bound, it is possible to use the socket for bidirectional read and write communication.",
        params(
            ("protocol" = String, Path, description = "IP transport protocol", example = "tcp"),
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

    let listener_id = ListenerId(protocol.into(), bind_host);
    if bind_host.port() > 0 && state.open_listeners.read_arc().await.contains_key(&listener_id) {
        return Err((StatusCode::CONFLICT, ApiErrorStatus::ListenHostAlreadyUsed));
    }

    debug!("binding {protocol} session listening socket to {bind_host}");
    let (bound_host, udp_session_id) = match protocol {
        IpProtocol::TCP => create_tcp_client_binding(bind_host, state.clone(), args.clone()).await?,
        IpProtocol::UDP => create_udp_client_binding(bind_host, state.clone(), args.clone()).await?,
    };

    Ok::<_, (StatusCode, ApiErrorStatus)>(
        (
            StatusCode::OK,
            Json(SessionClientResponse {
                protocol,
                ip: bound_host.ip().to_string(),
                port: bound_host.port(),
                target: args.target.to_string(),
                destination: args.destination,
                forward_path: args.forward_path.clone(),
                return_path: args.return_path.clone(),
                mtu: SESSION_MTU,
                surb_len: SURB_SIZE,
                active_clients: udp_session_id.into_iter().map(|s| s.to_string()).collect(),
            }),
        )
            .into_response(),
    )
}

/// Lists existing Session listeners for the given IP protocol.
#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/session/{{protocol}}"),
    description = "Lists existing Session listeners for the given IP protocol.",
    params(
        ("protocol" = String, Path, description = "IP transport protocol", example = "tcp"),
    ),
    responses(
        (status = 200, description = "Opened session listeners for the given IP protocol.", body = Vec<SessionClientResponse>, example = json!([
            {
                "target": "example.com:80",
                "destination": "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F",
                "forwardPath": { "Hops": 1 },
                "returnPath": { "Hops": 1 },
                "protocol": "tcp",
                "ip": "127.0.0.1",
                "port": 5542,
                "surbLen": 400,
                "mtu": 1020,
                "activeClients": []
            }
        ])),
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
        .read_arc()
        .await
        .iter()
        .filter(|(id, _)| id.0 == protocol.into())
        .map(|(id, entry)| SessionClientResponse {
            protocol,
            ip: id.1.ip().to_string(),
            port: id.1.port(),
            target: entry.target.to_string(),
            forward_path: entry.forward_path.clone(),
            return_path: entry.return_path.clone(),
            destination: entry.destination,
            mtu: SESSION_MTU,
            surb_len: SURB_SIZE,
            active_clients: entry.clients.iter().map(|e| e.key().to_string()).collect(),
        })
        .collect::<Vec<_>>();

    Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::OK, Json(response)).into_response())
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({
        "responseBuffer": "2 MB",
        "maxSurbUpstream": "2 Mbps"
    }))]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionConfig {
    /// The amount of response data the Session counterparty can deliver back to us,
    /// without us sending any SURBs to them.
    ///
    /// In other words, this size is recalculated to a number of SURBs delivered
    /// to the counterparty upfront and then maintained.
    /// The maintenance is dynamic, based on the number of responses we receive.
    ///
    /// All syntaxes like "2 MB", "128 kiB", "3MiB" are supported. The value must be
    /// at least the size of 2 Session packet payloads.
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = String)]
    pub response_buffer: Option<bytesize::ByteSize>,
    /// The maximum throughput at which artificial SURBs might be generated and sent
    /// to the recipient of the Session.
    ///
    /// On Sessions that rarely send data but receive a lot (= Exit node has high SURB consumption),
    /// this should roughly match the maximum retrieval throughput.
    ///
    /// All syntaxes like "2 MBps", "1.2Mbps", "300 kb/s", "1.23 Mb/s" are supported.
    #[serde(default)]
    #[serde(with = "human_bandwidth::option")]
    #[schema(value_type = String)]
    pub max_surb_upstream: Option<human_bandwidth::re::bandwidth::Bandwidth>,
}

impl From<SessionConfig> for Option<SurbBalancerConfig> {
    fn from(value: SessionConfig) -> Self {
        match value.response_buffer {
            // Buffer worth at least 2 reply packets
            Some(buffer_size) if buffer_size.as_u64() >= 2 * SESSION_MTU as u64 => Some(SurbBalancerConfig {
                target_surb_buffer_size: buffer_size.as_u64() / SESSION_MTU as u64,
                max_surbs_per_sec: value
                    .max_surb_upstream
                    .map(|b| (b.as_bps() as usize / (8 * SURB_SIZE)) as u64)
                    .unwrap_or_else(|| SurbBalancerConfig::default().max_surbs_per_sec),
                ..Default::default()
            }),
            // No additional SURBs are set up and maintained, useful for high-send low-reply sessions
            Some(_) => None,
            // Use defaults otherwise
            None => Some(SurbBalancerConfig::default()),
        }
    }
}

impl From<SurbBalancerConfig> for SessionConfig {
    fn from(value: SurbBalancerConfig) -> Self {
        Self {
            response_buffer: Some(bytesize::ByteSize::b(
                value.target_surb_buffer_size * SESSION_MTU as u64,
            )),
            max_surb_upstream: Some(human_bandwidth::re::bandwidth::Bandwidth::from_bps(
                value.max_surbs_per_sec * (8 * SURB_SIZE) as u64,
            )),
        }
    }
}

#[utoipa::path(
    post,
    path = const_format::formatcp!("{BASE_PATH}/session/config/{{id}}"),
    description = "Updates configuration of an existing active session.",
    params(
        ("id" = String, Path, description = "Session ID", example = "0x5112D584a1C72Fc25017:487"),
    ),
    request_body(
            content = SessionConfig,
            description = "Allows updating of several parameters of an existing active session.",
            content_type = "application/json"),
    responses(
            (status = 204, description = "Successfully updated the configuration"),
            (status = 400, description = "Invalid configuration.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Given session ID does not refer to an existing Session", body = ApiError),
            (status = 406, description = "Session cannot be reconfigured.", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError),
    ),
    security(
            ("api_token" = []),
            ("bearer_token" = [])
    ),
    tag = "Session"
)]
pub(crate) async fn adjust_session(
    State(state): State<Arc<InternalState>>,
    Path(session_id): Path<String>,
    Json(args): Json<SessionConfig>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let session_id = HoprSessionId::from_str(&session_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidSessionId))?;

    if let Some(cfg) = Option::<SurbBalancerConfig>::from(args) {
        match state.hopr.update_session_surb_balancer_config(&session_id, cfg).await {
            Ok(_) => Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::NO_CONTENT, "").into_response()),
            Err(HoprLibError::TransportError(HoprTransportError::Session(TransportSessionError::Manager(
                SessionManagerError::NonExistingSession,
            )))) => Err((StatusCode::NOT_FOUND, ApiErrorStatus::SessionNotFound)),
            Err(e) => Err((
                StatusCode::NOT_ACCEPTABLE,
                ApiErrorStatus::UnknownFailure(e.to_string()),
            )),
        }
    } else {
        Err::<_, (StatusCode, ApiErrorStatus)>((StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidInput))
    }
}

#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/session/config/{{id}}"),
    description = "Gets configuration of an existing active session.",
    params(
        ("id" = String, Path, description = "Session ID", example = "0x5112D584a1C72Fc25017:487"),
    ),
    responses(
            (status = 200, description = "Retrieved session configuration.", body = SessionConfig),
            (status = 400, description = "Invalid session ID.", body = ApiError),
            (status = 401, description = "Invalid authorization token.", body = ApiError),
            (status = 404, description = "Given session ID does not refer to an existing Session", body = ApiError),
            (status = 422, description = "Unknown failure", body = ApiError),
    ),
    security(
            ("api_token" = []),
            ("bearer_token" = [])
    ),
    tag = "Session"
)]
pub(crate) async fn session_config(
    State(state): State<Arc<InternalState>>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let session_id = HoprSessionId::from_str(&session_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidSessionId))?;

    match state.hopr.get_session_surb_balancer_config(&session_id).await {
        Ok(Some(cfg)) => {
            Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::OK, Json(SessionConfig::from(cfg))).into_response())
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, ApiErrorStatus::SessionNotFound)),
        Err(e) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            ApiErrorStatus::UnknownFailure(e.to_string()),
        )),
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display, strum::EnumString, utoipa::ToSchema,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[serde(rename_all = "lowercase")]
#[schema(example = "tcp")]
/// IP transport protocol
pub enum IpProtocol {
    #[allow(clippy::upper_case_acronyms)]
    TCP,
    #[allow(clippy::upper_case_acronyms)]
    UDP,
}

impl From<IpProtocol> for hopr_lib::IpProtocol {
    fn from(protocol: IpProtocol) -> hopr_lib::IpProtocol {
        match protocol {
            IpProtocol::TCP => hopr_lib::IpProtocol::TCP,
            IpProtocol::UDP => hopr_lib::IpProtocol::UDP,
        }
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
pub struct SessionCloseClientQuery {
    #[serde_as(as = "DisplayFromStr")]
    #[schema(value_type = String, example = "tcp")]
    /// IP transport protocol
    pub protocol: IpProtocol,

    /// Listening IP address of the Session.
    #[schema(example = "127.0.0.1:8545")]
    pub ip: String,

    /// Session port used for the listener.
    #[schema(value_type = u16, example = 10101)]
    pub port: u16,
}

/// Closes an existing Session listener.
/// The listener must've been previously created and bound for the given IP protocol.
/// Once a listener is closed, no more socket connections can be made to it.
/// If the passed port number is 0, listeners on all ports of the given listening IP and protocol
/// will be closed.
#[utoipa::path(
    delete,
    path = const_format::formatcp!("{BASE_PATH}/session/{{protocol}}/{{ip}}/{{port}}"),
    description = "Closes an existing Session listener.",
    params(SessionCloseClientQuery),
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
    Path(SessionCloseClientQuery { protocol, ip, port }): Path<SessionCloseClientQuery>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let listening_ip: IpAddr = ip
        .parse()
        .map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidInput))?;

    {
        let mut open_listeners = state.open_listeners.write_arc().await;

        let mut to_remove = Vec::new();

        // Find all listeners with protocol, listening IP and optionally port number (if > 0)
        open_listeners
            .iter()
            .filter(|(ListenerId(proto, addr), _)| {
                let protocol: hopr_lib::IpProtocol = protocol.into();
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

            entry.abort_handle.abort();
        }
    }

    Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::NO_CONTENT, "").into_response())
}

async fn try_restricted_bind<F, S, Fut>(
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

async fn bind_session_to_stream<T>(
    mut session: HoprSession,
    mut stream: T,
    max_buf: usize,
    abort_reg: Option<AbortRegistration>,
) where
    T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let session_id = *session.id();
    match transfer_session(&mut session, &mut stream, max_buf, abort_reg).await {
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
    use anyhow::Context;
    use futures::{
        FutureExt, StreamExt,
        channel::mpsc::{UnboundedReceiver, UnboundedSender},
    };
    use futures_time::future::FutureExt as TimeFutureExt;
    use hopr_crypto_types::crypto_traits::Randomizable;
    use hopr_lib::{ApplicationData, HoprPseudonym};
    use hopr_network_types::prelude::DestinationRouting;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    fn loopback_transport() -> (
        UnboundedSender<(DestinationRouting, ApplicationData)>,
        UnboundedReceiver<ApplicationData>,
    ) {
        let (input_tx, input_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationData)>();
        let (output_tx, output_rx) = futures::channel::mpsc::unbounded::<ApplicationData>();
        tokio::task::spawn(
            input_rx
                .map(|(_, data)| Ok(data))
                .forward(output_tx)
                .map(|e| tracing::debug!(?e, "loopback transport completed")),
        );

        (input_tx, output_rx)
    }

    #[tokio::test]
    async fn hoprd_session_connection_should_create_a_working_tcp_socket_through_which_data_can_be_sent_and_received()
    -> anyhow::Result<()> {
        let session_id = hopr_lib::HoprSessionId::new(4567u64, HoprPseudonym::random());
        let peer: hopr_lib::Address = "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F".parse()?;
        let session = hopr_lib::HoprSession::new(
            session_id,
            hopr_lib::DestinationRouting::forward_only(
                peer,
                hopr_lib::RoutingOptions::IntermediatePath(Default::default()),
            ),
            None,
            loopback_transport(),
            None,
        )?;

        let (bound_addr, tcp_listener) = tcp_listen_on(("127.0.0.1", 0)).await.context("listen_on failed")?;

        tokio::task::spawn(async move {
            match tcp_listener.accept().await {
                Ok((stream, _)) => bind_session_to_stream(session, stream, HOPR_TCP_BUFFER_SIZE, None).await,
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

    #[test_log::test(tokio::test)]
    async fn hoprd_session_connection_should_create_a_working_udp_socket_through_which_data_can_be_sent_and_received()
    -> anyhow::Result<()> {
        let session_id = hopr_lib::HoprSessionId::new(4567u64, HoprPseudonym::random());
        let peer: hopr_lib::Address = "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F".parse()?;
        let session = hopr_lib::HoprSession::new(
            session_id,
            hopr_lib::DestinationRouting::forward_only(
                peer,
                hopr_lib::RoutingOptions::IntermediatePath(Default::default()),
            ),
            None,
            loopback_transport(),
            None,
        )?;

        let (listen_addr, udp_listener) = udp_bind_to(("127.0.0.1", 0)).await.context("udp_bind_to failed")?;

        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let jh = tokio::task::spawn(bind_session_to_stream(
            session,
            udp_listener,
            ApplicationData::PAYLOAD_SIZE,
            Some(abort_registration),
        ));

        let mut udp_stream = ConnectedUdpStream::builder()
            .with_buffer_size(ApplicationData::PAYLOAD_SIZE)
            .with_queue_size(HOPR_UDP_QUEUE_SIZE)
            .with_counterparty(listen_addr)
            .build(("127.0.0.1", 0))
            .context("bind failed")?;

        let data = vec![b"hello", b"world", b"this ", b"is   ", b"    a", b" test"];

        for d in data.clone().into_iter() {
            udp_stream.write_all(d).await.context("write failed")?;
            // ConnectedUdpStream performs flush with each write
        }

        for d in data.iter() {
            let mut buf = vec![0; d.len()];
            udp_stream.read_exact(&mut buf).await.context("read failed")?;
        }

        // Once aborted, the bind_session_to_stream task must terminate too
        abort_handle.abort();
        jh.timeout(futures_time::time::Duration::from_millis(200)).await??;

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
