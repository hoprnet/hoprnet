use std::{fmt::Formatter, hash::Hash, net::IpAddr, str::FromStr, sync::Arc};

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
use futures::{AsyncReadExt, AsyncWriteExt, SinkExt, StreamExt};
use futures_concurrency::stream::Merge;
#[cfg(feature = "explicit-path")]
use hopr_lib::NodeId;
use hopr_lib::{
    Address, HoprSession, SESSION_MTU, SURB_SIZE, ServiceId, SessionAckMode, SessionCapabilities, SessionClientConfig,
    SessionId, SessionLifecycleState, SessionManagerError, SessionMetricsSnapshot, SessionTarget, SurbBalancerConfig,
    TransportSessionError,
    errors::{HoprLibError, HoprTransportError},
    utils::futures::AsyncReadStreamer,
};
use hopr_utils_session::{ListenerId, build_binding_host, create_tcp_client_binding, create_udp_client_binding};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, error, info, trace};

use crate::{ApiError, ApiErrorStatus, BASE_PATH, InternalState};

/// Size of the buffer for forwarding data to/from a TCP stream.
pub const HOPR_TCP_BUFFER_SIZE: usize = 4096;

/// Size of the buffer for forwarding data to/from a UDP stream.
pub const HOPR_UDP_BUFFER_SIZE: usize = 16384;

/// Size of the queue (back-pressure) for data incoming from a UDP stream.
pub const HOPR_UDP_QUEUE_SIZE: usize = 8192;

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

impl From<SessionTargetSpec> for hopr_utils_session::SessionTargetSpec {
    fn from(spec: SessionTargetSpec) -> Self {
        match spec {
            SessionTargetSpec::Plain(t) => Self::Plain(t),
            SessionTargetSpec::Sealed(t) => Self::Sealed(t),
            SessionTargetSpec::Service(t) => Self::Service(t),
        }
    }
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

/// Request parameters for creating a websocket session.
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
    /// Converts the websocket session request into protocol-level session configuration.
    pub(crate) async fn into_protocol_session_config(
        self,
    ) -> Result<(Address, SessionTarget, SessionClientConfig), ApiErrorStatus> {
        #[cfg(not(feature = "explicit-path"))]
        let path_options = hopr_lib::RoutingOptions::Hops((self.hops as u32).try_into()?);

        #[cfg(feature = "explicit-path")]
        let path_options = if let Some(path) = self.path {
            // Explicit `path` will override `hops`
            let path = path.into_iter().map(NodeId::from).collect::<Vec<_>>();
            hopr_lib::RoutingOptions::IntermediatePath(path.try_into()?)
        } else {
            hopr_lib::RoutingOptions::Hops((self.hops as u32).try_into()?)
        };

        let mut capabilities = SessionCapabilities::empty();
        capabilities.extend(self.capabilities.into_iter().flat_map(SessionCapabilities::from));

        let target_spec: hopr_utils_session::SessionTargetSpec = self.target.into();

        Ok((
            self.destination,
            target_spec.into_target(self.protocol.into())?,
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

/// Incoming input from either the network session or the websocket client.
enum WebSocketInput {
    /// Data received from the network session.
    Network(Result<Box<[u8]>, std::io::Error>),
    /// Message received from the websocket client.
    WsInput(Result<Message, Error>),
}

/// The maximum number of bytes read from a Session that WS can transfer within a single message.
const WS_MAX_SESSION_READ_SIZE: usize = 4096;

/// Manages bidirectional data forwarding between a websocket client and a network session.
///
/// This function bridges communication by receiving data from either source and forwarding
/// to the other, until either side closes the connection or an error occurs.
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

/// Routing options for the Session.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[schema(example = json!({ "Hops": 1 }))]
pub enum RoutingOptions {
    #[cfg(feature = "explicit-path")]
    #[schema(value_type = Vec<String>)]
    IntermediatePath(#[serde_as(as = "Vec<DisplayFromStr>")] Vec<NodeId>),
    Hops(usize),
}

impl RoutingOptions {
    /// Converts the API routing options into protocol-level routing options.
    pub(crate) async fn resolve(self) -> Result<hopr_lib::RoutingOptions, ApiErrorStatus> {
        Ok(match self {
            #[cfg(feature = "explicit-path")]
            RoutingOptions::IntermediatePath(path) => hopr_lib::RoutingOptions::IntermediatePath(path.try_into()?),
            RoutingOptions::Hops(hops) => hopr_lib::RoutingOptions::Hops(hops.try_into()?),
        })
    }
}

impl From<hopr_lib::RoutingOptions> for RoutingOptions {
    fn from(opts: hopr_lib::RoutingOptions) -> Self {
        match opts {
            hopr_lib::RoutingOptions::IntermediatePath(path) => {
                #[cfg(feature = "explicit-path")]
                {
                    RoutingOptions::IntermediatePath(path.into_iter().collect())
                }
                #[cfg(not(feature = "explicit-path"))]
                {
                    RoutingOptions::Hops(path.as_ref().len())
                }
            }
            hopr_lib::RoutingOptions::Hops(hops) => RoutingOptions::Hops(usize::from(hops)),
        }
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
    #[schema(value_type = Option<String>)]
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
    #[schema(value_type = Option<String>)]
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
    /// Converts the API client session request into protocol-level session configuration.
    pub(crate) async fn into_protocol_session_config(
        self,
        target_protocol: IpProtocol,
    ) -> Result<(hopr_lib::Address, SessionTarget, SessionClientConfig), ApiErrorStatus> {
        let target_spec: hopr_utils_session::SessionTargetSpec = self.target.clone().into();
        Ok((
            self.destination,
            target_spec.into_target(target_protocol.into())?,
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
        "target": "example.com:80",
        "destination": "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F",
        "forwardPath": { "Hops": 1 },
        "returnPath": { "Hops": 1 },
        "protocol": "tcp",
        "ip": "127.0.0.1",
        "port": 5542,
        "hoprMtu": 1002,
        "surbLen": 398,
        "activeClients": [],
        "maxClientSessions": 2,
        "maxSurbUpstream": "2000 kb/s",
        "responseBuffer": "2 MB",
        "sessionPool": 0
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
    /// MTU used by the underlying HOPR transport.
    pub hopr_mtu: usize,
    /// Size of a Single Use Reply Block used by the protocol.
    ///
    /// This is useful for SURB balancing calculations.
    pub surb_len: usize,
    /// Lists Session IDs of all active clients.
    ///
    /// Can contain multiple entries on TCP sessions, but currently
    /// always only a single entry on UDP sessions.
    pub active_clients: Vec<String>,
    /// The maximum number of client sessions that the listener can spawn.
    ///
    /// This currently applies only to the TCP sessions, as UDP sessions cannot
    /// have multiple clients (defaults to 1 for UDP).
    pub max_client_sessions: usize,
    /// The maximum throughput at which artificial SURBs might be generated and sent
    /// to the recipient of the Session.    
    #[serde(default)]
    #[serde(with = "human_bandwidth::option")]
    #[schema(value_type = Option<String>)]
    pub max_surb_upstream: Option<human_bandwidth::re::bandwidth::Bandwidth>,
    /// The amount of response data the Session counterparty can deliver back to us, without us
    /// sending any SURBs to them.
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[schema(value_type = Option<String>)]
    pub response_buffer: Option<bytesize::ByteSize>,
    /// How many Sessions to pool for clients.
    pub session_pool: Option<usize>,
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
    if bind_host.port() > 0 && state.open_listeners.0.contains_key(&listener_id) {
        return Err((StatusCode::CONFLICT, ApiErrorStatus::ListenHostAlreadyUsed));
    }

    let port_range = std::env::var(crate::env::HOPRD_SESSION_PORT_RANGE).ok();
    debug!("binding {protocol} session listening socket to {bind_host} (port range limitations: {port_range:?})");

    let (bound_host, udp_session_id, max_clients) = match protocol {
        IpProtocol::TCP => {
            let session_pool = args.session_pool;
            let max_client_sessions = args.max_client_sessions;
            let target_spec: hopr_utils_session::SessionTargetSpec = args.target.clone().into();
            let (destination, _target, config) = args
                .clone()
                .into_protocol_session_config(IpProtocol::TCP)
                .await
                .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;

            create_tcp_client_binding(
                bind_host,
                port_range,
                state.hopr.clone(),
                state.open_listeners.clone(),
                destination,
                target_spec,
                config,
                session_pool,
                max_client_sessions,
            )
            .await
            .map_err(|e| match e {
                hopr_utils_session::BindError::ListenHostAlreadyUsed => {
                    (StatusCode::CONFLICT, ApiErrorStatus::ListenHostAlreadyUsed)
                }
                hopr_utils_session::BindError::UnknownFailure(_) => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(format!("failed to start TCP listener on {bind_host}: {e}")),
                ),
            })?
        }
        IpProtocol::UDP => {
            let target_spec: hopr_utils_session::SessionTargetSpec = args.target.clone().into();
            let (destination, _target, config) = args
                .clone()
                .into_protocol_session_config(IpProtocol::UDP)
                .await
                .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e))?;

            create_udp_client_binding(
                bind_host,
                port_range,
                state.hopr.clone(),
                state.open_listeners.clone(),
                destination,
                target_spec,
                config,
            )
            .await
            .map_err(|e| match e {
                hopr_utils_session::BindError::ListenHostAlreadyUsed => {
                    (StatusCode::CONFLICT, ApiErrorStatus::ListenHostAlreadyUsed)
                }
                hopr_utils_session::BindError::UnknownFailure(_) => (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ApiErrorStatus::UnknownFailure(format!("failed to start UDP listener on {bind_host}: {e}")),
                ),
            })?
        }
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
                hopr_mtu: SESSION_MTU,
                surb_len: SURB_SIZE,
                active_clients: udp_session_id.into_iter().map(|s| s.to_string()).collect(),
                max_client_sessions: max_clients,
                max_surb_upstream: args.max_surb_upstream,
                response_buffer: args.response_buffer,
                session_pool: args.session_pool,
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
                "hoprMtu": 1020,
                "activeClients": [],
                "maxClientSessions": 2,
                "maxSurbUpstream": "2000 kb/s",
                "responseBuffer": "2 MB",
                "sessionPool": 0
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
        .0
        .iter()
        .filter(|v| v.key().0 == protocol.into())
        .map(|v| {
            let ListenerId(_, addr) = *v.key();
            let entry = v.value();
            SessionClientResponse {
                protocol,
                ip: addr.ip().to_string(),
                port: addr.port(),
                target: entry.target.to_string(),
                forward_path: entry.forward_path.clone().into(),
                return_path: entry.return_path.clone().into(),
                destination: entry.destination,
                hopr_mtu: SESSION_MTU,
                surb_len: SURB_SIZE,
                active_clients: entry.get_clients().iter().map(|e| e.key().to_string()).collect(),
                max_client_sessions: entry.max_client_sessions,
                max_surb_upstream: entry.max_surb_upstream,
                response_buffer: entry.response_buffer,
                session_pool: entry.session_pool,
            }
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
    /// Converts the API session config into protocol-level SURB balancer config.
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
    /// Converts protocol-level SURB balancer config into the API session config format.
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
    let session_id =
        SessionId::from_str(&session_id).map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidSessionId))?;

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
    let session_id =
        SessionId::from_str(&session_id).map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidSessionId))?;

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

/// Session lifecycle state for metrics.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SessionStatsState {
    /// Session is active and running.
    Active,
    /// Session is in the process of closing.
    Closing,
    /// Session has been fully closed.
    Closed,
}

impl From<SessionLifecycleState> for SessionStatsState {
    /// Converts protocol-level lifecycle state into the API metrics state format.
    fn from(value: SessionLifecycleState) -> Self {
        match value {
            SessionLifecycleState::Active => SessionStatsState::Active,
            SessionLifecycleState::Closing => SessionStatsState::Closing,
            SessionLifecycleState::Closed => SessionStatsState::Closed,
        }
    }
}

/// Session acknowledgement mode for metrics.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SessionStatsAckMode {
    /// No acknowledgements.
    None,
    /// Partial acknowledgements.
    Partial,
    /// Full acknowledgements.
    Full,
    /// Both (if applicable).
    Both,
}

impl From<SessionAckMode> for SessionStatsAckMode {
    /// Converts protocol-level acknowledgement mode into the API metrics mode format.
    fn from(value: SessionAckMode) -> Self {
        match value {
            SessionAckMode::None => SessionStatsAckMode::None,
            SessionAckMode::Partial => SessionStatsAckMode::Partial,
            SessionAckMode::Full => SessionStatsAckMode::Full,
            SessionAckMode::Both => SessionStatsAckMode::Both,
        }
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
/// Session lifetime metrics.
pub(crate) struct SessionStatsLifetime {
    /// Time when the session was created (in milliseconds since UNIX epoch).
    pub created_at_ms: u64,
    /// Time of the last read or write activity (in milliseconds since UNIX epoch).
    pub last_activity_at_ms: u64,
    /// Total duration the session has been alive (in milliseconds).
    pub uptime_ms: u64,
    /// Duration since the last activity (in milliseconds).
    pub idle_ms: u64,
    /// Current lifecycle state of the session.
    pub state: SessionStatsState,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
/// Session frame buffer metrics.
pub(crate) struct SessionStatsFrameBuffer {
    /// Maximum Transmission Unit for frames.
    pub frame_mtu: usize,
    /// Configured timeout for frame reassembly/acknowledgement (in milliseconds).
    pub frame_timeout_ms: u64,
    /// Configured capacity of the frame buffer.
    pub frame_capacity: usize,
    /// Number of frames currently being assembled (incomplete).
    pub incomplete_frames: usize,
    /// Total number of frames successfully completed/assembled.
    pub frames_completed: u64,
    /// Total number of frames emitted to the application.
    pub frames_emitted: u64,
    /// Total number of frames discarded (e.g. due to timeout or errors).
    pub frames_discarded: u64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
/// Session acknowledgement metrics.
pub(crate) struct SessionStatsAck {
    /// Configured acknowledgement mode.
    pub mode: SessionStatsAckMode,
    /// Total incoming segments received.
    pub incoming_segments: u64,
    /// Total outgoing segments sent.
    pub outgoing_segments: u64,
    /// Total retransmission requests received.
    pub retransmission_requests: u64,
    /// Total frames acknowledged by the peer.
    pub acknowledged_frames: u64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
/// Session SURB (Single Use Reply Block) metrics.
pub(crate) struct SessionStatsSurb {
    /// Total SURBs produced/minted.
    pub produced_total: u64,
    /// Total SURBs consumed/used.
    pub consumed_total: u64,
    /// Estimated number of SURBs currently available.
    pub buffer_estimate: u64,
    /// Target number of SURBs to maintain in buffer (if configured).
    pub target_buffer: Option<u64>,
    /// Rate of SURB consumption/production per second.
    pub rate_per_sec: f64,
    /// Whether a SURB refill request is currently in flight.
    pub refill_in_flight: bool,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
/// Session transport-level metrics.
pub(crate) struct SessionStatsTransport {
    /// Total bytes received.
    pub bytes_in: u64,
    /// Total bytes sent.
    pub bytes_out: u64,
    /// Total packets received.
    pub packets_in: u64,
    /// Total packets sent.
    pub packets_out: u64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
/// Complete snapshot of session metrics.
pub(crate) struct SessionStatsResponse {
    /// The session ID.
    pub session_id: String,
    /// Time when this snapshot was taken (in milliseconds since UNIX epoch).
    pub snapshot_at_ms: u64,
    /// Lifetime metrics.
    pub lifetime: SessionStatsLifetime,
    /// Frame buffer metrics.
    pub frame_buffer: SessionStatsFrameBuffer,
    /// Acknowledgement metrics.
    pub ack: SessionStatsAck,
    /// SURB metrics.
    pub surb: SessionStatsSurb,
    /// Transport metrics.
    pub transport: SessionStatsTransport,
}

impl From<SessionMetricsSnapshot> for SessionStatsResponse {
    /// Converts protocol-level metrics snapshot into the API response format.
    fn from(value: SessionMetricsSnapshot) -> Self {
        Self {
            session_id: value.session_id.to_string(),
            snapshot_at_ms: value
                .snapshot_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            lifetime: SessionStatsLifetime {
                created_at_ms: value
                    .lifetime
                    .created_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                last_activity_at_ms: value
                    .lifetime
                    .last_activity_at
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                uptime_ms: value.lifetime.uptime.as_millis() as u64,
                idle_ms: value.lifetime.idle.as_millis() as u64,
                state: value.lifetime.state.into(),
            },
            frame_buffer: SessionStatsFrameBuffer {
                frame_mtu: value.frame_buffer.frame_mtu,
                frame_timeout_ms: value.frame_buffer.frame_timeout.as_millis() as u64,
                frame_capacity: value.frame_buffer.frame_capacity,
                incomplete_frames: value.frame_buffer.incomplete_frames,
                frames_completed: value.frame_buffer.frames_completed,
                frames_emitted: value.frame_buffer.frames_emitted,
                frames_discarded: value.frame_buffer.frames_discarded,
            },
            ack: SessionStatsAck {
                mode: value.ack.mode.into(),
                incoming_segments: value.ack.incoming_segments,
                outgoing_segments: value.ack.outgoing_segments,
                retransmission_requests: value.ack.retransmission_requests,
                acknowledged_frames: value.ack.acknowledged_frames,
            },
            surb: SessionStatsSurb {
                produced_total: value.surb.produced_total,
                consumed_total: value.surb.consumed_total,
                buffer_estimate: value.surb.buffer_estimate,
                target_buffer: value.surb.target_buffer,
                rate_per_sec: value.surb.rate_per_sec,
                refill_in_flight: value.surb.refill_in_flight,
            },
            transport: SessionStatsTransport {
                bytes_in: value.transport.bytes_in,
                bytes_out: value.transport.bytes_out,
                packets_in: value.transport.packets_in,
                packets_out: value.transport.packets_out,
            },
        }
    }
}

#[utoipa::path(
    get,
    path = const_format::formatcp!("{BASE_PATH}/session/stats/{{id}}"),
    description = "Gets stats for an existing active session.",
    params(
        ("id" = String, Path, description = "Session ID", example = "0x5112D584a1C72Fc25017:487"),
    ),
    responses(
            (status = 200, description = "Retrieved session stats.", body = SessionStatsResponse),
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
pub(crate) async fn session_stats(
    State(state): State<Arc<InternalState>>,
    Path(session_id): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let session_id =
        SessionId::from_str(&session_id).map_err(|_| (StatusCode::BAD_REQUEST, ApiErrorStatus::InvalidSessionId))?;

    match state.hopr.get_session_stats(&session_id).await {
        Ok(metrics) => Ok::<_, (StatusCode, ApiErrorStatus)>(
            (StatusCode::OK, Json(SessionStatsResponse::from(metrics))).into_response(),
        ),
        Err(HoprLibError::TransportError(HoprTransportError::Session(TransportSessionError::Manager(
            SessionManagerError::NonExistingSession,
        )))) => Err((StatusCode::NOT_FOUND, ApiErrorStatus::SessionNotFound)),
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
        let open_listeners = &state.open_listeners.0;

        let mut to_remove = Vec::new();
        let protocol: hopr_lib::IpProtocol = protocol.into();

        // Find all listeners with protocol, listening IP and optionally port number (if > 0)
        open_listeners
            .iter()
            .filter(|v| {
                let ListenerId(proto, addr) = v.key();
                protocol == *proto && addr.ip() == listening_ip && (addr.port() == port || port == 0)
            })
            .for_each(|v| to_remove.push(*v.key()));

        if to_remove.is_empty() {
            return Err((StatusCode::NOT_FOUND, ApiErrorStatus::InvalidInput));
        }

        for bound_addr in to_remove {
            let (_, entry) = open_listeners
                .remove(&bound_addr)
                .ok_or((StatusCode::NOT_FOUND, ApiErrorStatus::InvalidInput))?;

            entry.abort_handle.abort();
        }
    }

    Ok::<_, (StatusCode, ApiErrorStatus)>((StatusCode::NO_CONTENT, "").into_response())
}
