//! Session-related utilities for HOPR
//!
//! This module provides utility functions and structures for managing sessions,
//! including session lifecycle management, session data handling, and common
//! session operations.

use std::{
    collections::{VecDeque},
    fmt::Formatter,
    future::Future,
    hash::Hash,
    net::SocketAddr,
    num::NonZeroUsize,
    str::FromStr,
    sync::Arc,
};

use base64::Engine;
use bytesize::ByteSize;
use dashmap::DashMap;
use futures::{
    FutureExt, StreamExt, TryStreamExt,
    future::{AbortHandle, AbortRegistration},
};
use hopr_network_types::{
    prelude::{ConnectedUdpStream, IpOrHost, IpProtocol, SealedHost, UdpStreamParallelism},
    udp::ForeignDataMode,
};
use hopr_transport::RoutingOptions;
use human_bandwidth::re::bandwidth::Bandwidth;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use tokio::net::TcpListener;
use tracing::{debug, error, info};
use hopr_chain_connector::HoprBlokliConnector;
use crate::{
    Address, Hopr, HoprSession, HoprSessionId, SURB_SIZE, ServiceId, SessionClientConfig, SessionTarget,
    errors::HoprLibError, transfer_session,
};
use crate::state::Abortable;

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Session target specification.
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

impl FromStr for SessionTargetSpec {
    type Err = HoprLibError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(stripped) = s.strip_prefix("$$") {
            Self::Sealed(
                base64::prelude::BASE64_URL_SAFE
                    .decode(stripped)
                    .map_err(|e| HoprLibError::Other(e.into()))?,
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
            (IpProtocol::TCP, SessionTargetSpec::Plain(plain)) => {
                SessionTarget::TcpStream(IpOrHost::from_str(&plain).map(SealedHost::from)?)
            }
            (IpProtocol::UDP, SessionTargetSpec::Plain(plain)) => {
                SessionTarget::UdpStream(IpOrHost::from_str(&plain).map(SealedHost::from)?)
            }
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
    /// The maximum number of client sessions that the listener can spawn.
    pub max_client_sessions: usize,
    /// The maximum number of SURB packets that can be sent upstream.
    pub max_surb_upstream: Option<human_bandwidth::re::bandwidth::Bandwidth>,
    /// The amount of response data the Session counterparty can deliver back to us, without us
    /// having to request it.
    pub response_buffer: Option<bytesize::ByteSize>,
    /// How many Sessions to pool for clients.
    pub session_pool: Option<usize>,
    /// The abort handle for the Session processing.
    pub abort_handle: AbortHandle,

    clients: Arc<DashMap<HoprSessionId, (SocketAddr, AbortHandle)>>,
}

impl StoredSessionEntry {
    pub fn get_clients(&self) -> &Arc<DashMap<HoprSessionId, (SocketAddr, AbortHandle)>> {
        &self.clients
    }
}

/// This function first tries to parse `requested` as the `ip:port` host pair.
/// If that does not work, it tries to parse `requested` as a single IP address
/// and as a `:` prefixed port number. Whichever of those fails, is replaced by the corresponding
/// part from the given `default`.
pub fn build_binding_host(requested: Option<&str>, default: std::net::SocketAddr) -> std::net::SocketAddr {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ListenerId(pub IpProtocol, pub std::net::SocketAddr);

impl std::fmt::Display for ListenerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}:{}", self.0, self.1.ip(), self.1.port())
    }
}

#[derive(Default)]
pub struct ListenerJoinHandles(pub DashMap<ListenerId, StoredSessionEntry>);

impl Abortable for ListenerJoinHandles {
    fn abort_process(&self) {
        self.0.alter_all(|_, v| {
            v.abort_handle.abort();
            v
        });
    }
}

pub struct SessionPool {
    pool: Option<Arc<std::sync::Mutex<VecDeque<HoprSession>>>>,
    ah: Option<AbortHandle>,
}

impl SessionPool {
    pub const MAX_SESSION_POOL_SIZE: usize = 5;

    pub async fn new(
        size: usize,
        dst: Address,
        target: SessionTarget,
        cfg: SessionClientConfig,
        hopr: Arc<Hopr<Arc<HoprBlokliConnector>>>,
    ) -> Result<Self, String> {
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
                            pool.lock().map_err(|_| "lock failed".to_string())?.push_back(s);
                            Ok(())
                        }
                        Err(error) => {
                            error!(%error, num_session = i, "failed to establish session for pool");
                            Err(format!("failed to establish session #{i} in pool to {dst}: {error}"))
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

    pub fn pop(&mut self) -> Option<HoprSession> {
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

#[allow(clippy::too_many_arguments)]
pub async fn create_tcp_client_binding(
    bind_host: std::net::SocketAddr,
    port_range: Option<String>,
    hopr: Arc<Hopr<Arc<HoprBlokliConnector>>>,
    open_listeners: Arc<ListenerJoinHandles>,
    destination: Address,
    target_spec: SessionTargetSpec,
    config: SessionClientConfig,
    use_session_pool: Option<usize>,
    max_client_sessions: Option<usize>,
) -> Result<(std::net::SocketAddr, Option<HoprSessionId>, usize), BindError> {
    // Bind the TCP socket first
    let (bound_host, tcp_listener) = tcp_listen_on(bind_host, port_range).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            BindError::ListenHostAlreadyUsed
        } else {
            BindError::UnknownFailure(format!("failed to start TCP listener on {bind_host}: {e}"))
        }
    })?;
    info!(%bound_host, "TCP session listener bound");

    // For each new TCP connection coming to the listener,
    // open a Session with the same parameters
    let target = target_spec
        .clone()
        .into_target(IpProtocol::TCP)
        .map_err(|e| BindError::UnknownFailure(e.to_string()))?;

    // Create a session pool if requested
    let session_pool_size = use_session_pool.unwrap_or(0);
    let mut session_pool = SessionPool::new(
        session_pool_size,
        destination,
        target.clone(),
        config.clone(),
        hopr.clone(),
    )
    .await
    .map_err(|e| BindError::UnknownFailure(e.to_string()))?;

    let active_sessions = Arc::new(DashMap::new());
    let mut max_clients = max_client_sessions.unwrap_or(5).max(1);

    if max_clients < session_pool_size {
        max_clients = session_pool_size;
    }

    let config_clone = config.clone();
    // Create an abort handler for the listener
    let (abort_handle, abort_reg) = AbortHandle::new_pair();
    let active_sessions_clone = active_sessions.clone();
    hopr_async_runtime::prelude::spawn(async move {
        let active_sessions_clone_2 = active_sessions_clone.clone();

        futures::stream::Abortable::new(tokio_stream::wrappers::TcpListenerStream::new(tcp_listener), abort_reg)
            .and_then(|sock| async { Ok((sock.peer_addr()?, sock)) })
            .for_each(move |accepted_client| {
                let data = config_clone.clone();
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
                                    match hopr.connect_to(destination, target, data).await {
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

    open_listeners.0.insert(
        ListenerId(hopr_network_types::types::IpProtocol::TCP, bound_host),
        StoredSessionEntry {
            destination,
            target: target_spec,
            forward_path: config.forward_path_options,
            return_path: config.return_path_options,
            clients: active_sessions,
            max_client_sessions: max_clients,
            max_surb_upstream: config
                .surb_management
                .map(|v| Bandwidth::from_bps(v.max_surbs_per_sec * SURB_SIZE as u64)),
            response_buffer: config
                .surb_management
                .map(|v| ByteSize::b(v.target_surb_buffer_size * SURB_SIZE as u64)),
            session_pool: Some(session_pool_size),
            abort_handle,
        },
    );
    Ok((bound_host, None, max_clients))
}

#[derive(Debug, thiserror::Error)]
pub enum BindError {
    #[error("conflict detected: listen host already in use")]
    ListenHostAlreadyUsed,

    #[error("unknown failure: {0}")]
    UnknownFailure(String),
}

pub async fn create_udp_client_binding(
    bind_host: std::net::SocketAddr,
    port_range: Option<String>,
    hopr: Arc<Hopr<Arc<HoprBlokliConnector>>>,
    open_listeners: Arc<ListenerJoinHandles>,
    destination: Address,
    target_spec: SessionTargetSpec,
    config: SessionClientConfig,
) -> Result<(std::net::SocketAddr, Option<HoprSessionId>, usize), BindError> {
    // Bind the UDP socket first
    let (bound_host, udp_socket) = udp_bind_to(bind_host, port_range).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            BindError::ListenHostAlreadyUsed
        } else {
            BindError::UnknownFailure(format!("failed to start UDP listener on {bind_host}: {e}"))
        }
    })?;

    info!(%bound_host, "UDP session listener bound");

    let target = target_spec
        .clone()
        .into_target(IpProtocol::UDP)
        .map_err(|e| BindError::UnknownFailure(e.to_string()))?;

    // Create a single session for the UDP socket
    let session = hopr
        .connect_to(destination, target, config.clone())
        .await
        .map_err(|e| BindError::UnknownFailure(e.to_string()))?;

    let open_listeners_clone = open_listeners.clone();
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
    let max_clients: usize = 1; // Maximum number of clients for this session. Currently always 1.

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
        open_listeners_clone.0.remove(&listener_id);
    });

    open_listeners.0.insert(
        listener_id,
        StoredSessionEntry {
            destination,
            target: target_spec,
            forward_path: config.forward_path_options.clone(),
            return_path: config.return_path_options.clone(),
            max_client_sessions: max_clients,
            max_surb_upstream: config
                .surb_management
                .map(|v| Bandwidth::from_bps(v.max_surbs_per_sec * SURB_SIZE as u64)),
            response_buffer: config
                .surb_management
                .map(|v| ByteSize::b(v.target_surb_buffer_size * SURB_SIZE as u64)),
            session_pool: None,
            abort_handle,
            clients,
        },
    );
    Ok((bound_host, Some(session_id), max_clients))
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

/// Listen on a specified address with a port from an optional port range for TCP connections.
async fn tcp_listen_on<A: std::net::ToSocketAddrs>(
    address: A,
    port_range: Option<String>,
) -> std::io::Result<(std::net::SocketAddr, TcpListener)> {
    let addrs = address.to_socket_addrs()?.collect::<Vec<_>>();

    // If automatic port allocation is requested and there's a restriction on the port range
    // (via HOPRD_SESSION_PORT_RANGE), try to find an address within that range.
    if addrs.iter().all(|a| a.port() == 0) {
        if let Some(range_str) = port_range {
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

pub async fn udp_bind_to<A: std::net::ToSocketAddrs>(
    address: A,
    port_range: Option<String>,
) -> std::io::Result<(std::net::SocketAddr, ConnectedUdpStream)> {
    let addrs = address.to_socket_addrs()?.collect::<Vec<_>>();

    let builder = ConnectedUdpStream::builder()
        .with_buffer_size(HOPR_UDP_BUFFER_SIZE)
        .with_foreign_data_mode(ForeignDataMode::Discard) // discard data from UDP clients other than the first one served
        .with_queue_size(HOPR_UDP_QUEUE_SIZE)
        .with_receiver_parallelism(
            std::env::var("HOPRD_SESSION_ENTRY_UDP_RX_PARALLELISM")
                .ok()
                .and_then(|s| s.parse::<NonZeroUsize>().ok())
                .map(UdpStreamParallelism::Specific)
                .unwrap_or(UdpStreamParallelism::Auto),
        );

    // If automatic port allocation is requested and there's a restriction on the port range
    // (via HOPRD_SESSION_PORT_RANGE), try to find an address within that range.
    if addrs.iter().all(|a| a.port() == 0) {
        if let Some(range_str) = port_range {
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::{
        Address, ApplicationData, ApplicationDataIn, ApplicationDataOut, DestinationRouting, HoprPseudonym,
        HoprSession, HoprSessionId, RoutingOptions,
    };

    fn loopback_transport() -> (
        UnboundedSender<(DestinationRouting, ApplicationDataOut)>,
        UnboundedReceiver<ApplicationDataIn>,
    ) {
        let (input_tx, input_rx) = futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let (output_tx, output_rx) = futures::channel::mpsc::unbounded::<ApplicationDataIn>();
        tokio::task::spawn(
            input_rx
                .map(|(_, data)| {
                    Ok(ApplicationDataIn {
                        data: data.data,
                        packet_info: Default::default(),
                    })
                })
                .forward(output_tx)
                .map(|e| tracing::debug!(?e, "loopback transport completed")),
        );

        (input_tx, output_rx)
    }

    #[tokio::test]
    async fn hoprd_session_connection_should_create_a_working_tcp_socket_through_which_data_can_be_sent_and_received()
    -> anyhow::Result<()> {
        let session_id = HoprSessionId::new(4567u64, HoprPseudonym::random());
        let peer: Address = "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F".parse()?;
        let session = HoprSession::new(
            session_id,
            DestinationRouting::forward_only(peer, RoutingOptions::IntermediatePath(Default::default())),
            Default::default(),
            loopback_transport(),
            None,
        )?;

        let (bound_addr, tcp_listener) = tcp_listen_on(("127.0.0.1", 0), None)
            .await
            .context("listen_on failed")?;

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
        let session_id = HoprSessionId::new(4567u64, HoprPseudonym::random());
        let peer: Address = "0x5112D584a1C72Fc250176B57aEba5fFbbB287D8F".parse()?;
        let session = HoprSession::new(
            session_id,
            DestinationRouting::forward_only(peer, RoutingOptions::IntermediatePath(Default::default())),
            Default::default(),
            loopback_transport(),
            None,
        )?;

        let (listen_addr, udp_listener) = udp_bind_to(("127.0.0.1", 0), None)
            .await
            .context("udp_bind_to failed")?;

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
    fn build_binding_address() {
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
